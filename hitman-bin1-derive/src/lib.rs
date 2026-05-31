use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{PathArguments, Type};

#[derive(FromMeta)]
#[darling(default, derive_syn_parse)]
struct Bin1RootAttrs {
	#[darling(rename = "crate", default = || syn::parse_str("::hitman_bin1_core").unwrap())]
	crate_path: syn::Path,

	alignment: Option<usize>
}

impl Default for Bin1RootAttrs {
	fn default() -> Self {
		Self {
			crate_path: syn::parse_str("::hitman_bin1_core").unwrap(),
			alignment: None
		}
	}
}

#[derive(Default, FromMeta)]
#[darling(default, derive_syn_parse)]
struct Bin1Attrs {
	#[darling(rename = "as")]
	as_type: Option<syn::Type>,

	pad: Option<usize>,
	pad_end: Option<usize>
}

#[proc_macro_derive(Bin1Serialize, attributes(bin1))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(input as syn::DeriveInput);

	let name = input.ident;

	let data = match input.data {
		syn::Data::Struct(data) => data,
		_ => panic!("Bin1Serialize can only be derived for structs")
	};

	let root_attrs = input
		.attrs
		.iter()
		.find_map(|attr| {
			attr.meta
				.path()
				.is_ident("bin1")
				.then(|| attr.parse_args::<Bin1RootAttrs>().unwrap())
		})
		.unwrap_or_default();

	let crate_path = root_attrs.crate_path;

	let field_types = data
		.fields
		.iter()
		.map(|f| {
			f.attrs
				.iter()
				.find_map(|attr| {
					attr.meta
						.path()
						.is_ident("bin1")
						.then(|| attr.parse_args::<Bin1Attrs>().unwrap())
				})
				.unwrap_or_default()
				.as_type
				.map(|ty| {
					let ty = match ty {
						Type::Path(path) => path,
						_ => panic!("Unexpected type")
					};

					let path_without_generics = {
						let mut path = ty.path.clone();
						if let Some(seg) = path.segments.last_mut() {
							seg.arguments = PathArguments::None;
						}
						path
					};

					let generics = ty
						.path
						.segments
						.last()
						.and_then(|seg| {
							match &seg.arguments {
								PathArguments::AngleBracketed(args) => Some(args),
								_ => None
							}
							.map(|args| quote! { #args })
						})
						.unwrap_or_default();

					quote! { #path_without_generics::Ser #generics }
				})
				.unwrap_or_else(|| {
					let ty = &f.ty;
					quote! { #ty }
				})
		})
		.collect::<Vec<_>>();

	// Align struct to maximum of members (repeated ifs so it's a valid const expression)
	let alignment = root_attrs.alignment.map_or_else(
		|| {
			let mut iter = field_types.iter().map(|ty| {
				quote! { <#ty as #crate_path::ser::Aligned>::ALIGNMENT }
			});

			let first = match iter.next() {
				Some(t) => quote! { let x = #t; },
				None => quote! { let x = 1usize; }
			};

			iter.fold(first, |acc, next| {
				quote! {
					#acc
					let x = if #next > x { #next } else { x };
				}
			})
		},
		|alignment| {
			quote! { let x = #alignment; }
		}
	);

	let write_fields = data.fields.iter().enumerate().fold(quote! {}, |acc, (idx, f)| {
		let field = f.ident.to_owned().unwrap();

		let options: Bin1Attrs = f
			.attrs
			.iter()
			.find_map(|attr| attr.meta.path().is_ident("bin1").then(|| attr.parse_args().unwrap()))
			.unwrap_or_default();

		let padding = options
			.pad
			.map(|padding| {
				quote! {
					ser.write_unaligned(&[0u8; #padding]);
				}
			})
			.unwrap_or_default();

		let padding_end = options
			.pad_end
			.map(|padding| {
				quote! {
					ser.write_unaligned(&[0u8; #padding]);
				}
			})
			.unwrap_or_default();

		let log = cfg_select! {
			feature = "debug-log" => {
				quote! {
					eprintln!("0x{:6X}: writing field {}::{}", ser.position(), stringify!(#name), stringify!(#field));
				}
			},
			_ => quote! {}
		};

		if options.as_type.is_some() {
			let as_type = &field_types[idx];
			quote! {
				#acc
				#log
				#padding
				#as_type::from(self.#field.as_ref()).write_aligned(ser)?;
				#padding_end
			}
		} else {
			quote! {
				#acc
				#log
				#padding
				self.#field.write_aligned(ser)?;
				#padding_end
			}
		}
	});

	let resolve_fields = data.fields.iter().enumerate().fold(quote! {}, |acc, (idx, f)| {
		let field = f.ident.to_owned().unwrap();

		let options: Bin1Attrs = f
			.attrs
			.iter()
			.find_map(|attr| attr.meta.path().is_ident("bin1").then(|| attr.parse_args().unwrap()))
			.unwrap_or_default();

		let log = cfg_select! {
			feature = "debug-log" => {
				quote! {
					eprintln!("0x{:6X}: resolving field {}::{}", ser.position(), stringify!(#name), stringify!(#field));
				}
			},
			_ => quote! {}
		};

		if options.as_type.is_some() {
			let as_type = &field_types[idx];
			quote! {
				#acc
				#log
				#as_type::from(self.#field.as_ref()).resolve(ser)?;
			}
		} else {
			quote! {
				#acc
				#log
				self.#field.resolve(ser)?;
			}
		}
	});

	let log_write = cfg_select! {
		feature = "debug-log" => {
			quote! {
				eprintln!("0x{:6X}: writing {}", ser.position(), stringify!(#name));
			}
		},
		_ => quote! {}
	};

	let log_resolve = cfg_select! {
		feature = "debug-log" => {
			quote! {
				eprintln!("0x{:6X}: resolving {}", ser.position(), stringify!(#name));
			}
		},
		_ => quote! {}
	};

	let expanded = quote! {
		impl #crate_path::ser::Aligned for #name {
			const ALIGNMENT: usize = { #alignment x };
		}

		impl #crate_path::ser::Bin1Serialize for #name {
			fn alignment(&self) -> usize {
				<Self as #crate_path::ser::Aligned>::ALIGNMENT
			}

			fn write(&self, ser: &mut #crate_path::ser::Bin1Serializer)
				-> Result<(), #crate_path::ser::SerializeError>
			{
				#log_write
				#write_fields
				ser.align_to(<Self as #crate_path::ser::Aligned>::ALIGNMENT);
				Ok(())
			}

			fn resolve(&self, ser: &mut #crate_path::ser::Bin1Serializer)
				-> Result<(), #crate_path::ser::SerializeError>
			{
				#log_resolve
				#resolve_fields
				Ok(())
			}
		}
	};

	TokenStream::from(expanded)
}

#[proc_macro_derive(Bin1Deserialize, attributes(bin1))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(input as syn::DeriveInput);

	let name = input.ident;

	let data = match input.data {
		syn::Data::Struct(data) => data,
		_ => panic!("Bin1Deserialize can only be derived for structs")
	};

	let root_attrs = input
		.attrs
		.iter()
		.find_map(|attr| {
			attr.meta
				.path()
				.is_ident("bin1")
				.then(|| attr.parse_args::<Bin1RootAttrs>().unwrap())
		})
		.unwrap_or_default();

	let crate_path = root_attrs.crate_path;

	let field_types = data
		.fields
		.iter()
		.map(|f| {
			f.attrs
				.iter()
				.find_map(|attr| {
					attr.meta
						.path()
						.is_ident("bin1")
						.then(|| attr.parse_args::<Bin1Attrs>().unwrap())
				})
				.unwrap_or_default()
				.as_type
				.map(|ty| {
					let ty = match ty {
						Type::Path(path) => path,
						_ => panic!("Unexpected type")
					};

					let path_without_generics = {
						let mut path = ty.path.clone();
						if let Some(seg) = path.segments.last_mut() {
							seg.arguments = PathArguments::None;
						}
						path
					};

					let generics = ty
						.path
						.segments
						.last()
						.and_then(|seg| {
							match &seg.arguments {
								PathArguments::AngleBracketed(args) => Some(args),
								_ => None
							}
							.map(|args| quote! { #args })
						})
						.unwrap_or_default();

					quote! { #path_without_generics::De #generics }
				})
				.unwrap_or_else(|| {
					let ty = &f.ty;
					quote! { #ty }
				})
		})
		.collect::<Vec<_>>();

	let mut size = {
		let mut size = quote! { let mut size = 0; };

		for (f, ty) in data.fields.iter().zip(field_types.iter()) {
			let options: Bin1Attrs = f
				.attrs
				.iter()
				.find_map(|attr| attr.meta.path().is_ident("bin1").then(|| attr.parse_args().unwrap()))
				.unwrap_or_default();

			let pad = options.pad.map(|x| quote! { + #x }).unwrap_or_default();
			let pad_end = options.pad_end.map(|x| quote! { + #x }).unwrap_or_default();

			let alignment_padding = quote! {
				(<#ty as #crate_path::ser::Aligned>::ALIGNMENT
					- (size % <#ty as #crate_path::ser::Aligned>::ALIGNMENT))
					% <#ty as #crate_path::ser::Aligned>::ALIGNMENT
			};

			size.extend(quote! {
				size += #alignment_padding #pad + <#ty as #crate_path::de::Bin1Deserialize>::SIZE #pad_end;
			});
		}

		size
	};

	let end_padding = quote! {
		(<Self as #crate_path::ser::Aligned>::ALIGNMENT
			- ({ #size size } % <Self as #crate_path::ser::Aligned>::ALIGNMENT))
			% <Self as #crate_path::ser::Aligned>::ALIGNMENT
	};

	size.extend(quote! {
		size += (<Self as #crate_path::ser::Aligned>::ALIGNMENT
			- (size % <Self as #crate_path::ser::Aligned>::ALIGNMENT))
			% <Self as #crate_path::ser::Aligned>::ALIGNMENT;
		size
	});

	let read_fields = data.fields.iter().enumerate().fold(quote! {}, |acc, (idx, f)| {
		let field = f.ident.to_owned().unwrap();

		let options: Bin1Attrs = f
			.attrs
			.iter()
			.find_map(|attr| attr.meta.path().is_ident("bin1").then(|| attr.parse_args().unwrap()))
			.unwrap_or_default();

		let padding = options
			.pad
			.map(|padding| {
				let padding = padding as i64;
				quote! {
					de.seek_relative(#padding)?;
				}
			})
			.unwrap_or_default();

		let padding_end = options
			.pad_end
			.map(|padding| {
				let padding = padding as i64;
				quote! {
					de.seek_relative(#padding)?;
				}
			})
			.unwrap_or_default();

		let log = cfg_select! {
			feature = "debug-log" => {
				quote! {
					eprintln!("0x{:6X}: reading field {}::{}", de.position(), stringify!(#name), stringify!(#field));
				}
			},
			_ => quote! {}
		};

		if options.as_type.is_some() {
			let as_type = &field_types[idx];
			quote! {
				#acc
				#log
				#padding
				de.align_to(<#as_type as #crate_path::ser::Aligned>::ALIGNMENT)?;
				let #field = #as_type::read(de)?.into();
				#padding_end
			}
		} else {
			let ty = &field_types[idx];
			quote! {
				#acc
				#log
				#padding
				de.align_to(<#ty as #crate_path::ser::Aligned>::ALIGNMENT)?;
				let #field = <#ty>::read(de)?;
				#padding_end
			}
		}
	});

	let fields = data.fields.iter().map(|f| {
		let field = f.ident.to_owned().unwrap();
		quote! {
			#field
		}
	});

	let log = cfg_select! {
		feature = "debug-log" => {
			quote! {
				eprintln!("0x{:6X}: deserializing {}", de.position(), stringify!(#name));
			}
		},
		_ => quote! {}
	};

	let expanded = quote! {
		impl #crate_path::de::Bin1Deserialize for #name {
			#[allow(clippy::modulo_one)]
			const SIZE: usize = { #size };

			fn read(de: &mut #crate_path::de::Bin1Deserializer)
				-> Result<Self, #crate_path::de::DeserializeError>
			{
				#log
				#read_fields
				de.seek_relative(const { #[allow(clippy::modulo_one)] { (#end_padding) as i64 } })?;
				Ok(Self { #(#fields),* })
			}
		}
	};

	TokenStream::from(expanded)
}
