use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
	#[error("failed to convert ZVariant value: {0}")]
	VariantConversion(#[from] serde_json::Error),

	#[error("this game version does not support pin connection overrides")]
	PinConnectionOverrideUnsupported,

	#[error("this game version does not support array exposed entities")]
	ArrayExposedEntityUnsupported,

	#[error("this game version does not support non-array exposed entities with multiple targets")]
	NonArrayExposedEntityHasMultipleTargets,

	#[error("this game version does not support constant pin values")]
	ConstantPinValueUnsupported
}

pub trait ConvertFrom<T>: Sized {
	type Error;

	fn convert_from(value: T) -> Result<Self, Self::Error>;
}

pub trait ConvertInto<T>: Sized {
	type Error;

	fn convert_into(self) -> Result<T, Self::Error>;
}

impl<T, U: ConvertFrom<T>> ConvertInto<U> for T {
	type Error = <U as ConvertFrom<T>>::Error;

	fn convert_into(self) -> Result<U, Self::Error> {
		U::convert_from(self)
	}
}

#[cfg(all(feature = "h1", feature = "h3"))]
mod h1_h3 {
	use tryvial::try_fn;

	use super::{super::*, ConversionError, ConvertFrom, ConvertInto};

	impl ConvertFrom<h1::STemplateEntity> for h3::STemplateEntityFactory {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::STemplateEntity) -> Result<Self, Self::Error> {
			Self {
				sub_type: value.sub_type,
				blueprint_index_in_resource_header: value.blueprint_index_in_resource_header,
				root_entity_index: value.root_entity_index,
				sub_entities: value
					.entity_templates
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				property_overrides: value
					.property_overrides
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				external_scene_type_indices_in_resource_header: value.external_scene_type_indices_in_resource_header
			}
		}
	}

	impl ConvertFrom<h1::STemplateSubEntity> for h3::STemplateFactorySubEntity {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::STemplateSubEntity) -> Result<Self, Self::Error> {
			Self {
				logical_parent: value.logical_parent.convert_into()?,
				entity_type_resource_index: value.entity_type_resource_index,
				property_values: value
					.property_values
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				post_init_property_values: value
					.post_init_property_values
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				platform_specific_property_values: vec![]
			}
		}
	}

	impl ConvertFrom<h1::SEntityTemplateReference> for h3::SEntityTemplateReference {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::SEntityTemplateReference) -> Result<Self, Self::Error> {
			Self {
				entity_id: value.entity_id,
				external_scene_index: value.external_scene_index,
				entity_index: value.entity_index,
				exposed_entity: value.exposed_entity
			}
		}
	}

	impl ConvertFrom<h1::SEntityTemplateProperty> for h3::SEntityTemplateProperty {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::SEntityTemplateProperty) -> Result<Self, Self::Error> {
			Self {
				property_id: value.property_id,
				value: serde_json::from_value(serde_json::to_value(&value.value)?)
					.unwrap_or_else(|_| value.value.into_inner().into())
			}
		}
	}

	impl ConvertFrom<h1::SEntityTemplatePropertyOverride> for h3::SEntityTemplatePropertyOverride {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::SEntityTemplatePropertyOverride) -> Result<Self, Self::Error> {
			Self {
				property_owner: value.property_owner.convert_into()?,
				property_value: value.property_value.convert_into()?
			}
		}
	}

	impl ConvertFrom<h1::STemplateEntityBlueprint> for h3::STemplateEntityBlueprint {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::STemplateEntityBlueprint) -> Result<Self, Self::Error> {
			Self {
				sub_type: value.sub_type,
				root_entity_index: value.root_entity_index,
				sub_entities: value
					.entity_templates
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				external_scene_type_indices_in_resource_header: value.external_scene_type_indices_in_resource_header,
				pin_connections: value
					.pin_connections
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				input_pin_forwardings: value
					.input_pin_forwardings
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				output_pin_forwardings: value
					.output_pin_forwardings
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				override_deletes: value
					.override_deletes
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				pin_connection_overrides: vec![],
				pin_connection_override_deletes: vec![]
			}
		}
	}

	impl ConvertFrom<h1::STemplateSubEntityBlueprint> for h3::STemplateBlueprintSubEntity {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::STemplateSubEntityBlueprint) -> Result<Self, Self::Error> {
			Self {
				logical_parent: value.logical_parent.convert_into()?,
				entity_type_resource_index: value.entity_type_resource_index,
				entity_id: value.entity_id,
				editor_only: false,
				entity_name: value.entity_name,
				property_aliases: value
					.property_aliases
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				exposed_entities: value
					.exposed_entities
					.into_iter()
					.map(|(name, entity)| {
						Ok::<_, ConversionError>(h3::SEntityTemplateExposedEntity {
							name,
							is_array: false,
							targets: vec![entity.convert_into()?]
						})
					})
					.collect::<Result<_, _>>()?,
				exposed_interfaces: value.exposed_interfaces,
				entity_subsets: value
					.entity_subsets
					.into_iter()
					.map(|(name, subset)| Ok::<_, ConversionError>((name, subset.convert_into()?)))
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h1::SEntityTemplatePropertyAlias> for h3::SEntityTemplatePropertyAlias {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::SEntityTemplatePropertyAlias) -> Result<Self, Self::Error> {
			Self {
				alias_name: value.alias_name,
				entity_id: value.entity_id,
				property_name: value.property_name
			}
		}
	}

	impl ConvertFrom<h1::SEntityTemplateEntitySubset> for h3::SEntityTemplateEntitySubset {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::SEntityTemplateEntitySubset) -> Result<Self, Self::Error> {
			Self {
				entities: value.entities
			}
		}
	}

	impl ConvertFrom<h1::SEntityTemplatePinConnection> for h3::SEntityTemplatePinConnection {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h1::SEntityTemplatePinConnection) -> Result<Self, Self::Error> {
			Self {
				from_id: value.from_id,
				to_id: value.to_id,
				from_pin_name: value.from_pin_name,
				to_pin_name: value.to_pin_name,
				constant_pin_value: h3::ZVariant::new(())
			}
		}
	}

	impl ConvertFrom<h3::STemplateEntityFactory> for h1::STemplateEntity {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::STemplateEntityFactory) -> Result<Self, Self::Error> {
			Self {
				sub_type: value.sub_type,
				blueprint_index_in_resource_header: value.blueprint_index_in_resource_header,
				root_entity_index: value.root_entity_index,
				entity_templates: value
					.sub_entities
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				property_overrides: value
					.property_overrides
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				external_scene_type_indices_in_resource_header: value.external_scene_type_indices_in_resource_header
			}
		}
	}

	impl ConvertFrom<h3::STemplateFactorySubEntity> for h1::STemplateSubEntity {
		type Error = ConversionError;

		/// Converts PC platform-specific properties into regular properties and ignores all other platform-specific properties.
		#[try_fn]
		fn convert_from(value: h3::STemplateFactorySubEntity) -> Result<Self, Self::Error> {
			let mut pc_init = vec![];
			let mut pc_post_init = vec![];

			for prop in value.platform_specific_property_values {
				if prop.platform == h3::EVirtualPlatformID::Pc {
					if prop.post_init {
						pc_post_init.push(prop.property_value);
					} else {
						pc_init.push(prop.property_value);
					}
				}
			}

			Self {
				logical_parent: value.logical_parent.convert_into()?,
				entity_type_resource_index: value.entity_type_resource_index,
				property_values: value
					.property_values
					.into_iter()
					.chain(pc_init)
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				post_init_property_values: value
					.post_init_property_values
					.into_iter()
					.chain(pc_post_init)
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplateReference> for h1::SEntityTemplateReference {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplateReference) -> Result<Self, Self::Error> {
			Self {
				entity_id: value.entity_id,
				external_scene_index: value.external_scene_index,
				entity_index: value.entity_index,
				exposed_entity: value.exposed_entity
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplateProperty> for h1::SEntityTemplateProperty {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplateProperty) -> Result<Self, Self::Error> {
			Self {
				property_id: value.property_id,
				value: serde_json::from_value(serde_json::to_value(&value.value)?)
					.unwrap_or_else(|_| value.value.into_inner().into())
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplatePropertyOverride> for h1::SEntityTemplatePropertyOverride {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplatePropertyOverride) -> Result<Self, Self::Error> {
			Self {
				property_owner: value.property_owner.convert_into()?,
				property_value: value.property_value.convert_into()?
			}
		}
	}

	impl ConvertFrom<h3::STemplateEntityBlueprint> for h1::STemplateEntityBlueprint {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::STemplateEntityBlueprint) -> Result<Self, Self::Error> {
			if !value.pin_connection_overrides.is_empty() || !value.pin_connection_override_deletes.is_empty() {
				return Err(ConversionError::PinConnectionOverrideUnsupported);
			}

			Self {
				sub_type: value.sub_type,
				root_entity_index: value.root_entity_index,
				entity_templates: value
					.sub_entities
					.into_iter()
					.filter(|x| !x.editor_only)
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				external_scene_type_indices_in_resource_header: value.external_scene_type_indices_in_resource_header,
				pin_connections: value
					.pin_connections
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				input_pin_forwardings: value
					.input_pin_forwardings
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				output_pin_forwardings: value
					.output_pin_forwardings
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				override_deletes: value
					.override_deletes
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h3::STemplateBlueprintSubEntity> for h1::STemplateSubEntityBlueprint {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::STemplateBlueprintSubEntity) -> Result<Self, Self::Error> {
			Self {
				logical_parent: value.logical_parent.convert_into()?,
				entity_type_resource_index: value.entity_type_resource_index,
				entity_id: value.entity_id,
				entity_name: value.entity_name,
				property_aliases: value
					.property_aliases
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				exposed_entities: value
					.exposed_entities
					.into_iter()
					.map(|mut entity| {
						if entity.is_array {
							Err(ConversionError::ArrayExposedEntityUnsupported)
						} else if entity.targets.len() != 1 {
							Err(ConversionError::NonArrayExposedEntityHasMultipleTargets)
						} else {
							Ok::<_, ConversionError>((entity.name, entity.targets.remove(0).convert_into()?))
						}
					})
					.collect::<Result<_, _>>()?,
				exposed_interfaces: value.exposed_interfaces,
				entity_subsets: value
					.entity_subsets
					.into_iter()
					.map(|(name, subset)| Ok::<_, ConversionError>((name, subset.convert_into()?)))
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplatePropertyAlias> for h1::SEntityTemplatePropertyAlias {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplatePropertyAlias) -> Result<Self, Self::Error> {
			Self {
				alias_name: value.alias_name,
				entity_id: value.entity_id,
				property_name: value.property_name
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplateEntitySubset> for h1::SEntityTemplateEntitySubset {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplateEntitySubset) -> Result<Self, Self::Error> {
			Self {
				entities: value.entities
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplatePinConnection> for h1::SEntityTemplatePinConnection {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplatePinConnection) -> Result<Self, Self::Error> {
			if !value.constant_pin_value.is::<()>() {
				return Err(ConversionError::ConstantPinValueUnsupported);
			}

			Self {
				from_id: value.from_id,
				to_id: value.to_id,
				from_pin_name: value.from_pin_name,
				to_pin_name: value.to_pin_name
			}
		}
	}
}

#[cfg(all(feature = "h2", feature = "h3"))]
mod h2_h3 {
	use tryvial::try_fn;

	use super::{super::*, ConversionError, ConvertFrom, ConvertInto};

	impl ConvertFrom<h2::STemplateEntityFactory> for h3::STemplateEntityFactory {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::STemplateEntityFactory) -> Result<Self, Self::Error> {
			Self {
				sub_type: value.sub_type,
				blueprint_index_in_resource_header: value.blueprint_index_in_resource_header,
				root_entity_index: value.root_entity_index,
				sub_entities: value
					.sub_entities
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				property_overrides: value
					.property_overrides
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				external_scene_type_indices_in_resource_header: value.external_scene_type_indices_in_resource_header
			}
		}
	}

	impl ConvertFrom<h2::STemplateFactorySubEntity> for h3::STemplateFactorySubEntity {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::STemplateFactorySubEntity) -> Result<Self, Self::Error> {
			Self {
				logical_parent: value.logical_parent.convert_into()?,
				entity_type_resource_index: value.entity_type_resource_index,
				property_values: value
					.property_values
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				post_init_property_values: value
					.post_init_property_values
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				platform_specific_property_values: vec![]
			}
		}
	}

	impl ConvertFrom<h2::SEntityTemplateReference> for h3::SEntityTemplateReference {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::SEntityTemplateReference) -> Result<Self, Self::Error> {
			Self {
				entity_id: value.entity_id,
				external_scene_index: value.external_scene_index,
				entity_index: value.entity_index,
				exposed_entity: value.exposed_entity
			}
		}
	}

	impl ConvertFrom<h2::SEntityTemplateProperty> for h3::SEntityTemplateProperty {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::SEntityTemplateProperty) -> Result<Self, Self::Error> {
			Self {
				property_id: value.property_id,
				value: serde_json::from_value(serde_json::to_value(&value.value)?)
					.unwrap_or_else(|_| value.value.into_inner().into())
			}
		}
	}

	impl ConvertFrom<h2::SEntityTemplatePropertyOverride> for h3::SEntityTemplatePropertyOverride {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::SEntityTemplatePropertyOverride) -> Result<Self, Self::Error> {
			Self {
				property_owner: value.property_owner.convert_into()?,
				property_value: value.property_value.convert_into()?
			}
		}
	}

	impl ConvertFrom<h2::STemplateEntityBlueprint> for h3::STemplateEntityBlueprint {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::STemplateEntityBlueprint) -> Result<Self, Self::Error> {
			Self {
				sub_type: value.sub_type,
				root_entity_index: value.root_entity_index,
				sub_entities: value
					.sub_entities
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				external_scene_type_indices_in_resource_header: value.external_scene_type_indices_in_resource_header,
				pin_connections: value
					.pin_connections
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				input_pin_forwardings: value
					.input_pin_forwardings
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				output_pin_forwardings: value
					.output_pin_forwardings
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				override_deletes: value
					.override_deletes
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				pin_connection_overrides: value
					.pin_connection_overrides
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				pin_connection_override_deletes: value
					.pin_connection_override_deletes
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h2::STemplateBlueprintSubEntity> for h3::STemplateBlueprintSubEntity {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::STemplateBlueprintSubEntity) -> Result<Self, Self::Error> {
			Self {
				logical_parent: value.logical_parent.convert_into()?,
				entity_type_resource_index: value.entity_type_resource_index,
				entity_id: value.entity_id,
				editor_only: value.editor_only,
				entity_name: value.entity_name,
				property_aliases: value
					.property_aliases
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				exposed_entities: value
					.exposed_entities
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				exposed_interfaces: value.exposed_interfaces,
				entity_subsets: value
					.entity_subsets
					.into_iter()
					.map(|(name, subset)| Ok::<_, ConversionError>((name, subset.convert_into()?)))
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h2::SEntityTemplatePropertyAlias> for h3::SEntityTemplatePropertyAlias {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::SEntityTemplatePropertyAlias) -> Result<Self, Self::Error> {
			Self {
				alias_name: value.alias_name,
				entity_id: value.entity_id,
				property_name: value.property_name
			}
		}
	}

	impl ConvertFrom<h2::SEntityTemplateExposedEntity> for h3::SEntityTemplateExposedEntity {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::SEntityTemplateExposedEntity) -> Result<Self, Self::Error> {
			Self {
				name: value.name,
				is_array: value.is_array,
				targets: value
					.targets
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h2::SEntityTemplateEntitySubset> for h3::SEntityTemplateEntitySubset {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::SEntityTemplateEntitySubset) -> Result<Self, Self::Error> {
			Self {
				entities: value.entities
			}
		}
	}

	impl ConvertFrom<h2::SEntityTemplatePinConnection> for h3::SEntityTemplatePinConnection {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::SEntityTemplatePinConnection) -> Result<Self, Self::Error> {
			Self {
				from_id: value.from_id,
				to_id: value.to_id,
				from_pin_name: value.from_pin_name,
				to_pin_name: value.to_pin_name,
				constant_pin_value: serde_json::from_value(serde_json::to_value(&value.constant_pin_value)?)
					.unwrap_or_else(|_| value.constant_pin_value.into_inner().into())
			}
		}
	}

	impl ConvertFrom<h2::SExternalEntityTemplatePinConnection> for h3::SExternalEntityTemplatePinConnection {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h2::SExternalEntityTemplatePinConnection) -> Result<Self, Self::Error> {
			Self {
				from_entity: value.from_entity.convert_into()?,
				to_entity: value.to_entity.convert_into()?,
				from_pin_name: value.from_pin_name,
				to_pin_name: value.to_pin_name,
				constant_pin_value: serde_json::from_value(serde_json::to_value(&value.constant_pin_value)?)
					.unwrap_or_else(|_| value.constant_pin_value.into_inner().into())
			}
		}
	}

	impl ConvertFrom<h3::STemplateEntityFactory> for h2::STemplateEntityFactory {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::STemplateEntityFactory) -> Result<Self, Self::Error> {
			Self {
				sub_type: value.sub_type,
				blueprint_index_in_resource_header: value.blueprint_index_in_resource_header,
				root_entity_index: value.root_entity_index,
				sub_entities: value
					.sub_entities
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				property_overrides: value
					.property_overrides
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				external_scene_type_indices_in_resource_header: value.external_scene_type_indices_in_resource_header
			}
		}
	}

	impl ConvertFrom<h3::STemplateFactorySubEntity> for h2::STemplateFactorySubEntity {
		type Error = ConversionError;

		/// Converts PC platform-specific properties into regular properties and ignores all other platform-specific properties.
		#[try_fn]
		fn convert_from(value: h3::STemplateFactorySubEntity) -> Result<Self, Self::Error> {
			let mut pc_init = vec![];
			let mut pc_post_init = vec![];

			for prop in value.platform_specific_property_values {
				if prop.platform == h3::EVirtualPlatformID::Pc {
					if prop.post_init {
						pc_post_init.push(prop.property_value);
					} else {
						pc_init.push(prop.property_value);
					}
				}
			}

			Self {
				logical_parent: value.logical_parent.convert_into()?,
				entity_type_resource_index: value.entity_type_resource_index,
				property_values: value
					.property_values
					.into_iter()
					.chain(pc_init)
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				post_init_property_values: value
					.post_init_property_values
					.into_iter()
					.chain(pc_post_init)
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplateReference> for h2::SEntityTemplateReference {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplateReference) -> Result<Self, Self::Error> {
			Self {
				entity_id: value.entity_id,
				external_scene_index: value.external_scene_index,
				entity_index: value.entity_index,
				exposed_entity: value.exposed_entity
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplateProperty> for h2::SEntityTemplateProperty {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplateProperty) -> Result<Self, Self::Error> {
			Self {
				property_id: value.property_id,
				value: serde_json::from_value(serde_json::to_value(&value.value)?)
					.unwrap_or_else(|_| value.value.into_inner().into())
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplatePropertyOverride> for h2::SEntityTemplatePropertyOverride {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplatePropertyOverride) -> Result<Self, Self::Error> {
			Self {
				property_owner: value.property_owner.convert_into()?,
				property_value: value.property_value.convert_into()?
			}
		}
	}

	impl ConvertFrom<h3::STemplateEntityBlueprint> for h2::STemplateEntityBlueprint {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::STemplateEntityBlueprint) -> Result<Self, Self::Error> {
			Self {
				sub_type: value.sub_type,
				root_entity_index: value.root_entity_index,
				sub_entities: value
					.sub_entities
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				external_scene_type_indices_in_resource_header: value.external_scene_type_indices_in_resource_header,
				pin_connections: value
					.pin_connections
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				input_pin_forwardings: value
					.input_pin_forwardings
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				output_pin_forwardings: value
					.output_pin_forwardings
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				override_deletes: value
					.override_deletes
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				pin_connection_overrides: value
					.pin_connection_overrides
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				pin_connection_override_deletes: value
					.pin_connection_override_deletes
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h3::STemplateBlueprintSubEntity> for h2::STemplateBlueprintSubEntity {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::STemplateBlueprintSubEntity) -> Result<Self, Self::Error> {
			Self {
				logical_parent: value.logical_parent.convert_into()?,
				entity_type_resource_index: value.entity_type_resource_index,
				entity_id: value.entity_id,
				editor_only: value.editor_only,
				entity_name: value.entity_name,
				property_aliases: value
					.property_aliases
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				exposed_entities: value
					.exposed_entities
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?,
				exposed_interfaces: value.exposed_interfaces,
				entity_subsets: value
					.entity_subsets
					.into_iter()
					.map(|(name, subset)| Ok::<_, ConversionError>((name, subset.convert_into()?)))
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplatePropertyAlias> for h2::SEntityTemplatePropertyAlias {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplatePropertyAlias) -> Result<Self, Self::Error> {
			Self {
				alias_name: value.alias_name,
				entity_id: value.entity_id,
				property_name: value.property_name
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplateExposedEntity> for h2::SEntityTemplateExposedEntity {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplateExposedEntity) -> Result<Self, Self::Error> {
			Self {
				name: value.name,
				is_array: value.is_array,
				targets: value
					.targets
					.into_iter()
					.map(|x| x.convert_into())
					.collect::<Result<_, _>>()?
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplateEntitySubset> for h2::SEntityTemplateEntitySubset {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplateEntitySubset) -> Result<Self, Self::Error> {
			Self {
				entities: value.entities
			}
		}
	}

	impl ConvertFrom<h3::SEntityTemplatePinConnection> for h2::SEntityTemplatePinConnection {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SEntityTemplatePinConnection) -> Result<Self, Self::Error> {
			Self {
				from_id: value.from_id,
				to_id: value.to_id,
				from_pin_name: value.from_pin_name,
				to_pin_name: value.to_pin_name,
				constant_pin_value: serde_json::from_value(serde_json::to_value(&value.constant_pin_value)?)
					.unwrap_or_else(|_| value.constant_pin_value.into_inner().into())
			}
		}
	}

	impl ConvertFrom<h3::SExternalEntityTemplatePinConnection> for h2::SExternalEntityTemplatePinConnection {
		type Error = ConversionError;

		#[try_fn]
		fn convert_from(value: h3::SExternalEntityTemplatePinConnection) -> Result<Self, Self::Error> {
			Self {
				from_entity: value.from_entity.convert_into()?,
				to_entity: value.to_entity.convert_into()?,
				from_pin_name: value.from_pin_name,
				to_pin_name: value.to_pin_name,
				constant_pin_value: serde_json::from_value(serde_json::to_value(&value.constant_pin_value)?)
					.unwrap_or_else(|_| value.constant_pin_value.into_inner().into())
			}
		}
	}
}
