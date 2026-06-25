import std.core;
import std.sys;
import std.limits;
import std.string;
import type.magic;

fn format_hex(auto value) {
    return std::format("0x{:X}", value);
};

fn format_ptr(auto value) {
    return std::format("pointer to 0x{:X}", value);
};

struct Bin1Header {
    type::Magic<"BIN1"> magic;
    padding[1];
    u8 ser_alignment;
    u8 segments;
    padding[1];
    be u32 size;
    padding[4];
};

struct RebasedPointer {
    u32 location;
    u64 rebased_pointer @ location + 0x10 [[color("ffffff"), format("format_ptr")]];
};

struct RebasedSegment {
    u32 count;
    RebasedPointer ptrs[count];
};

struct TypeID {
    u32 idx;
    u32 unk;
    std::assert(unk == std::limits::u32_max(), "unknown TypeID value is always max");
    u32 len;
    char name[len];
    
    padding[($ % 4 == 0) ? 0 : (4 - ($ % 4))];
};

struct TypeIDsSegment {
    u32 ptrs_count;
    u32 ptrs[ptrs_count];
    
    u32 ids_count;
    TypeID ids[ids_count];
};

struct RuntimeResourceIDsSegment {
    u32 count;
    u32 ptrs[count];
};

struct ResourcePtrsSegment {
    u32 count;
    u32 ptrs[count];
};

struct Bin1Segment {
    u32 id [[format("format_hex")]];
    u32 len;
    
    match (id) {
        (0x12EBA5ED): RebasedSegment data;
        (0x3989BF9F): TypeIDsSegment data;
        (0x578FBCEE): RuntimeResourceIDsSegment data;
        (0x64603664): ResourcePtrsSegment data;
        (_): u8 data[len];
    }
} [[format("format_segment")]];

fn format_segment(ref Bin1Segment value) {
    return std::format("{:X} ({} bytes)", value.id, value.len);
};

struct Bin1File<T> {
    Bin1Header header;
    Bin1Segment segments[header.segments] @ 0x10 + header.size;
    
    T data @ 0x10;
};

struct TArray<T> {
    u64 alloc_end_flags = std::mem::read_unsigned($ + 16, 8);
    if (alloc_end_flags == std::limits::u64_max()) {
        u64 start [[format("format_ptr")]];
        u64 end [[format("format_ptr")]];
        u64 alloc_end [[format("format_ptr")]];
        T items[0];
    } else if (alloc_end_flags & 0x4000000000000000 != 0) {
        u8 len = alloc_end_flags & 0xFF;
        T items[len];
    } else {
        u64 start [[format("format_ptr")]];
        u64 end [[format("format_ptr")]];
        u64 alloc_end [[format("format_ptr")]];
        
        T items[while($ != end + 0x10)] @ start + 0x10;
        if (std::core::member_count(items) == std::mem::read_unsigned(start + 0x10 - 4, 4)) {
            u32 len_prefix @ start + 0x10 - 4;
        }
    }
} [[format("format_tarray")]];

fn format_tarray(ref auto arr) {
    if (typenameof(arr) == "TArray<u8>") {
        return std::format("u8[{}]", std::core::member_count(arr.items));
    } else {
        return std::format("{}[{}] {}", typenameof(arr.items), std::core::member_count(arr.items), arr.items);
    }
};

struct ZString {
    u32 len [[format("format_strlen")]];
    padding[4];
    u64 ptr [[format("format_ptr")]];

    char data [(len & ~0x40000000) + 1] @ ptr + 0x10;
    if ((len & ~0x40000000) + 1 == std::mem::read_unsigned(ptr + 0x10 - 4, 4)) {
        u32 len_prefix @ ptr + 0x10 - 4;
    }
} [[format("format_zstring")]];

fn format_strlen(u32 len) {
    return std::format("{}", len & ~0x40000000);
};

fn format_zstring(ref ZString string) {
    return std::format("{}", string.data);
};
