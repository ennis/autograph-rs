use gl;
use gl::types::*;

#[derive(Copy, Clone, Debug)]
pub enum TextureDimensions
{
    Tex1D,
    Tex2D,
    Tex3D,
    Tex1DArray,
    Tex2DArray,
    TexCube
}

#[derive(Copy, Clone, Debug)]
// These are actually Vulkan formats
pub enum TextureFormat {
    UNDEFINED = 0,
    R4G4_UNORM_PACK8 = 1,
    R4G4B4A4_UNORM_PACK16 = 2,
    B4G4R4A4_UNORM_PACK16 = 3,
    R5G6B5_UNORM_PACK16 = 4,
    B5G6R5_UNORM_PACK16 = 5,
    R5G5B5A1_UNORM_PACK16 = 6,
    B5G5R5A1_UNORM_PACK16 = 7,
    A1R5G5B5_UNORM_PACK16 = 8,
    R8_UNORM = 9,
    R8_SNORM = 10,
    R8_USCALED = 11,
    R8_SSCALED = 12,
    R8_UINT = 13,
    R8_SINT = 14,
    R8_SRGB = 15,
    R8G8_UNORM = 16,
    R8G8_SNORM = 17,
    R8G8_USCALED = 18,
    R8G8_SSCALED = 19,
    R8G8_UINT = 20,
    R8G8_SINT = 21,
    R8G8_SRGB = 22,
    R8G8B8_UNORM = 23,
    R8G8B8_SNORM = 24,
    R8G8B8_USCALED = 25,
    R8G8B8_SSCALED = 26,
    R8G8B8_UINT = 27,
    R8G8B8_SINT = 28,
    R8G8B8_SRGB = 29,
    B8G8R8_UNORM = 30,
    B8G8R8_SNORM = 31,
    B8G8R8_USCALED = 32,
    B8G8R8_SSCALED = 33,
    B8G8R8_UINT = 34,
    B8G8R8_SINT = 35,
    B8G8R8_SRGB = 36,
    R8G8B8A8_UNORM = 37,
    R8G8B8A8_SNORM = 38,
    R8G8B8A8_USCALED = 39,
    R8G8B8A8_SSCALED = 40,
    R8G8B8A8_UINT = 41,
    R8G8B8A8_SINT = 42,
    R8G8B8A8_SRGB = 43,
    B8G8R8A8_UNORM = 44,
    B8G8R8A8_SNORM = 45,
    B8G8R8A8_USCALED = 46,
    B8G8R8A8_SSCALED = 47,
    B8G8R8A8_UINT = 48,
    B8G8R8A8_SINT = 49,
    B8G8R8A8_SRGB = 50,
    A8B8G8R8_UNORM_PACK32 = 51,
    A8B8G8R8_SNORM_PACK32 = 52,
    A8B8G8R8_USCALED_PACK32 = 53,
    A8B8G8R8_SSCALED_PACK32 = 54,
    A8B8G8R8_UINT_PACK32 = 55,
    A8B8G8R8_SINT_PACK32 = 56,
    A8B8G8R8_SRGB_PACK32 = 57,
    A2R10G10B10_UNORM_PACK32 = 58,
    A2R10G10B10_SNORM_PACK32 = 59,
    A2R10G10B10_USCALED_PACK32 = 60,
    A2R10G10B10_SSCALED_PACK32 = 61,
    A2R10G10B10_UINT_PACK32 = 62,
    A2R10G10B10_SINT_PACK32 = 63,
    A2B10G10R10_UNORM_PACK32 = 64,
    A2B10G10R10_SNORM_PACK32 = 65,
    A2B10G10R10_USCALED_PACK32 = 66,
    A2B10G10R10_SSCALED_PACK32 = 67,
    A2B10G10R10_UINT_PACK32 = 68,
    A2B10G10R10_SINT_PACK32 = 69,
    R16_UNORM = 70,
    R16_SNORM = 71,
    R16_USCALED = 72,
    R16_SSCALED = 73,
    R16_UINT = 74,
    R16_SINT = 75,
    R16_SFLOAT = 76,
    R16G16_UNORM = 77,
    R16G16_SNORM = 78,
    R16G16_USCALED = 79,
    R16G16_SSCALED = 80,
    R16G16_UINT = 81,
    R16G16_SINT = 82,
    R16G16_SFLOAT = 83,
    R16G16B16_UNORM = 84,
    R16G16B16_SNORM = 85,
    R16G16B16_USCALED = 86,
    R16G16B16_SSCALED = 87,
    R16G16B16_UINT = 88,
    R16G16B16_SINT = 89,
    R16G16B16_SFLOAT = 90,
    R16G16B16A16_UNORM = 91,
    R16G16B16A16_SNORM = 92,
    R16G16B16A16_USCALED = 93,
    R16G16B16A16_SSCALED = 94,
    R16G16B16A16_UINT = 95,
    R16G16B16A16_SINT = 96,
    R16G16B16A16_SFLOAT = 97,
    R32_UINT = 98,
    R32_SINT = 99,
    R32_SFLOAT = 100,
    R32G32_UINT = 101,
    R32G32_SINT = 102,
    R32G32_SFLOAT = 103,
    R32G32B32_UINT = 104,
    R32G32B32_SINT = 105,
    R32G32B32_SFLOAT = 106,
    R32G32B32A32_UINT = 107,
    R32G32B32A32_SINT = 108,
    R32G32B32A32_SFLOAT = 109,
    R64_UINT = 110,
    R64_SINT = 111,
    R64_SFLOAT = 112,
    R64G64_UINT = 113,
    R64G64_SINT = 114,
    R64G64_SFLOAT = 115,
    R64G64B64_UINT = 116,
    R64G64B64_SINT = 117,
    R64G64B64_SFLOAT = 118,
    R64G64B64A64_UINT = 119,
    R64G64B64A64_SINT = 120,
    R64G64B64A64_SFLOAT = 121,
    B10G11R11_UFLOAT_PACK32 = 122,
    E5B9G9R9_UFLOAT_PACK32 = 123,
    D16_UNORM = 124,
    X8_D24_UNORM_PACK32 = 125,
    D32_SFLOAT = 126,
    S8_UINT = 127,
    D16_UNORM_S8_UINT = 128,
    D24_UNORM_S8_UINT = 129,
    D32_SFLOAT_S8_UINT = 130,
    BC1_RGB_UNORM_BLOCK = 131,
    BC1_RGB_SRGB_BLOCK = 132,
    BC1_RGBA_UNORM_BLOCK = 133,
    BC1_RGBA_SRGB_BLOCK = 134,
    BC2_UNORM_BLOCK = 135,
    BC2_SRGB_BLOCK = 136,
    BC3_UNORM_BLOCK = 137,
    BC3_SRGB_BLOCK = 138,
    BC4_UNORM_BLOCK = 139,
    BC4_SNORM_BLOCK = 140,
    BC5_UNORM_BLOCK = 141,
    BC5_SNORM_BLOCK = 142,
    BC6H_UFLOAT_BLOCK = 143,
    BC6H_SFLOAT_BLOCK = 144,
    BC7_UNORM_BLOCK = 145,
    BC7_SRGB_BLOCK = 146,
    ETC2_R8G8B8_UNORM_BLOCK = 147,
    ETC2_R8G8B8_SRGB_BLOCK = 148,
    ETC2_R8G8B8A1_UNORM_BLOCK = 149,
    ETC2_R8G8B8A1_SRGB_BLOCK = 150,
    ETC2_R8G8B8A8_UNORM_BLOCK = 151,
    ETC2_R8G8B8A8_SRGB_BLOCK = 152,
    EAC_R11_UNORM_BLOCK = 153,
    EAC_R11_SNORM_BLOCK = 154,
    EAC_R11G11_UNORM_BLOCK = 155,
    EAC_R11G11_SNORM_BLOCK = 156,
    ASTC_4x4_UNORM_BLOCK = 157,
    ASTC_4x4_SRGB_BLOCK = 158,
    ASTC_5x4_UNORM_BLOCK = 159,
    ASTC_5x4_SRGB_BLOCK = 160,
    ASTC_5x5_UNORM_BLOCK = 161,
    ASTC_5x5_SRGB_BLOCK = 162,
    ASTC_6x5_UNORM_BLOCK = 163,
    ASTC_6x5_SRGB_BLOCK = 164,
    ASTC_6x6_UNORM_BLOCK = 165,
    ASTC_6x6_SRGB_BLOCK = 166,
    ASTC_8x5_UNORM_BLOCK = 167,
    ASTC_8x5_SRGB_BLOCK = 168,
    ASTC_8x6_UNORM_BLOCK = 169,
    ASTC_8x6_SRGB_BLOCK = 170,
    ASTC_8x8_UNORM_BLOCK = 171,
    ASTC_8x8_SRGB_BLOCK = 172,
    ASTC_10x5_UNORM_BLOCK = 173,
    ASTC_10x5_SRGB_BLOCK = 174,
    ASTC_10x6_UNORM_BLOCK = 175,
    ASTC_10x6_SRGB_BLOCK = 176,
    ASTC_10x8_UNORM_BLOCK = 177,
    ASTC_10x8_SRGB_BLOCK = 178,
    ASTC_10x10_UNORM_BLOCK = 179,
    ASTC_10x10_SRGB_BLOCK = 180,
    ASTC_12x10_UNORM_BLOCK = 181,
    ASTC_12x10_SRGB_BLOCK = 182,
    ASTC_12x12_UNORM_BLOCK = 183,
    ASTC_12x12_SRGB_BLOCK = 184,
}

pub enum ComponentLayout
{
    UNKNOWN,
    R,
    RG,
    RGB,
    RGBA,
    BGR,
    BGRA,
    ARGB,
    ABGR,
    EBGR,
    D,
    DS,
    S,
    XD
}

pub enum TextureFormatType
{
    UNKNOWN,
    UNORM,
    SNORM,
    USCALED,
    SSCALED,
    UINT,
    SINT,
    SRGB,
    UFLOAT,
    SFLOAT,
    UNORM_UINT,
    SFLOAT_UINT
}

pub struct TextureFormatInfo
{
    pub component_layout: ComponentLayout,
    pub component_bits: [u8; 4],
    pub format_type: TextureFormatType
}

impl TextureFormatInfo
{
    pub fn is_compressed(&self) -> bool {
        self.component_bits == [0,0,0,0]
    }

    pub fn byte_size(&self) -> usize {
        (self.component_bits[0] + self.component_bits[1] + self.component_bits[2] + self.component_bits[3]) as usize / 8
    }
}

static TF_UNDEFINED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::UNKNOWN, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNKNOWN };
static TF_R4G4_UNORM_PACK8: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [4, 4, 0, 0], format_type: TextureFormatType::UNORM };
static TF_R4G4B4A4_UNORM_PACK16: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [4, 4, 4, 4], format_type: TextureFormatType::UNORM };
static TF_B4G4R4A4_UNORM_PACK16: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGRA, component_bits: [4, 4, 4, 4], format_type: TextureFormatType::UNORM };
static TF_R5G6B5_UNORM_PACK16: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [5, 6, 5, 0], format_type: TextureFormatType::UNORM };
static TF_B5G6R5_UNORM_PACK16: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGR, component_bits: [5, 6, 5, 0], format_type: TextureFormatType::UNORM };
static TF_R5G5B5A1_UNORM_PACK16: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [5, 5, 5, 1], format_type: TextureFormatType::UNORM };
static TF_B5G5R5A1_UNORM_PACK16: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGRA, component_bits: [5, 5, 5, 1], format_type: TextureFormatType::UNORM };
static TF_A1R5G5B5_UNORM_PACK16: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ARGB, component_bits: [1, 5, 5, 5], format_type: TextureFormatType::UNORM };
static TF_R8_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [8, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_R8_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [8, 0, 0, 0], format_type: TextureFormatType::SNORM };
static TF_R8_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [8, 0, 0, 0], format_type: TextureFormatType::USCALED };
static TF_R8_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [8, 0, 0, 0], format_type: TextureFormatType::SSCALED };
static TF_R8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [8, 0, 0, 0], format_type: TextureFormatType::UINT };
static TF_R8_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [8, 0, 0, 0], format_type: TextureFormatType::SINT };
static TF_R8_SRGB: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [8, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_R8G8_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [8, 8, 0, 0], format_type: TextureFormatType::UNORM };
static TF_R8G8_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [8, 8, 0, 0], format_type: TextureFormatType::SNORM };
static TF_R8G8_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [8, 8, 0, 0], format_type: TextureFormatType::USCALED };
static TF_R8G8_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [8, 8, 0, 0], format_type: TextureFormatType::SSCALED };
static TF_R8G8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [8, 8, 0, 0], format_type: TextureFormatType::UINT };
static TF_R8G8_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [8, 8, 0, 0], format_type: TextureFormatType::SINT };
static TF_R8G8_SRGB: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [8, 8, 0, 0], format_type: TextureFormatType::SRGB };
static TF_R8G8B8_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::UNORM };
static TF_R8G8B8_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::SNORM };
static TF_R8G8B8_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::USCALED };
static TF_R8G8B8_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::SSCALED };
static TF_R8G8B8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::UINT };
static TF_R8G8B8_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::SINT };
static TF_R8G8B8_SRGB: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::SRGB };
static TF_B8G8R8_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGR, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::UNORM };
static TF_B8G8R8_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGR, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::SNORM };
static TF_B8G8R8_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGR, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::USCALED };
static TF_B8G8R8_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGR, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::SSCALED };
static TF_B8G8R8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGR, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::UINT };
static TF_B8G8R8_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGR, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::SINT };
static TF_B8G8R8_SRGB: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGR, component_bits: [8, 8, 8, 0], format_type: TextureFormatType::SRGB };
static TF_R8G8B8A8_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::UNORM };
static TF_R8G8B8A8_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SNORM };
static TF_R8G8B8A8_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::USCALED };
static TF_R8G8B8A8_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SSCALED };
static TF_R8G8B8A8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::UINT };
static TF_R8G8B8A8_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SINT };
static TF_R8G8B8A8_SRGB: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SRGB };
static TF_B8G8R8A8_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGRA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::UNORM };
static TF_B8G8R8A8_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGRA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SNORM };
static TF_B8G8R8A8_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGRA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::USCALED };
static TF_B8G8R8A8_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGRA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SSCALED };
static TF_B8G8R8A8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGRA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::UINT };
static TF_B8G8R8A8_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGRA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SINT };
static TF_B8G8R8A8_SRGB: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGRA, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SRGB };
static TF_A8B8G8R8_UNORM_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::UNORM };
static TF_A8B8G8R8_SNORM_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SNORM };
static TF_A8B8G8R8_USCALED_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::USCALED };
static TF_A8B8G8R8_SSCALED_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SSCALED };
static TF_A8B8G8R8_UINT_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::UINT };
static TF_A8B8G8R8_SINT_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SINT };
static TF_A8B8G8R8_SRGB_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [8, 8, 8, 8], format_type: TextureFormatType::SRGB };
static TF_A2R10G10B10_UNORM_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ARGB, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::UNORM };
static TF_A2R10G10B10_SNORM_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ARGB, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::SNORM };
static TF_A2R10G10B10_USCALED_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ARGB, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::USCALED };
static TF_A2R10G10B10_SSCALED_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ARGB, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::SSCALED };
static TF_A2R10G10B10_UINT_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ARGB, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::UINT };
static TF_A2R10G10B10_SINT_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ARGB, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::SINT };
static TF_A2B10G10R10_UNORM_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::UNORM };
static TF_A2B10G10R10_SNORM_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::SNORM };
static TF_A2B10G10R10_USCALED_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::USCALED };
static TF_A2B10G10R10_SSCALED_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::SSCALED };
static TF_A2B10G10R10_UINT_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::UINT };
static TF_A2B10G10R10_SINT_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::ABGR, component_bits: [2, 10, 10, 10], format_type: TextureFormatType::SINT };
static TF_R16_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [16, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_R16_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [16, 0, 0, 0], format_type: TextureFormatType::SNORM };
static TF_R16_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [16, 0, 0, 0], format_type: TextureFormatType::USCALED };
static TF_R16_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [16, 0, 0, 0], format_type: TextureFormatType::SSCALED };
static TF_R16_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [16, 0, 0, 0], format_type: TextureFormatType::UINT };
static TF_R16_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [16, 0, 0, 0], format_type: TextureFormatType::SINT };
static TF_R16_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [16, 0, 0, 0], format_type: TextureFormatType::SFLOAT };
static TF_R16G16_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [16, 16, 0, 0], format_type: TextureFormatType::UNORM };
static TF_R16G16_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [16, 16, 0, 0], format_type: TextureFormatType::SNORM };
static TF_R16G16_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [16, 16, 0, 0], format_type: TextureFormatType::USCALED };
static TF_R16G16_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [16, 16, 0, 0], format_type: TextureFormatType::SSCALED };
static TF_R16G16_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [16, 16, 0, 0], format_type: TextureFormatType::UINT };
static TF_R16G16_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [16, 16, 0, 0], format_type: TextureFormatType::SINT };
static TF_R16G16_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [16, 16, 0, 0], format_type: TextureFormatType::SFLOAT };
static TF_R16G16B16_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [16, 16, 16, 0], format_type: TextureFormatType::UNORM };
static TF_R16G16B16_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [16, 16, 16, 0], format_type: TextureFormatType::SNORM };
static TF_R16G16B16_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [16, 16, 16, 0], format_type: TextureFormatType::USCALED };
static TF_R16G16B16_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [16, 16, 16, 0], format_type: TextureFormatType::SSCALED };
static TF_R16G16B16_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [16, 16, 16, 0], format_type: TextureFormatType::UINT };
static TF_R16G16B16_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [16, 16, 16, 0], format_type: TextureFormatType::SINT };
static TF_R16G16B16_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [16, 16, 16, 0], format_type: TextureFormatType::SFLOAT };
static TF_R16G16B16A16_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [16, 16, 16, 16], format_type: TextureFormatType::UNORM };
static TF_R16G16B16A16_SNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [16, 16, 16, 16], format_type: TextureFormatType::SNORM };
static TF_R16G16B16A16_USCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [16, 16, 16, 16], format_type: TextureFormatType::USCALED };
static TF_R16G16B16A16_SSCALED: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [16, 16, 16, 16], format_type: TextureFormatType::SSCALED };
static TF_R16G16B16A16_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [16, 16, 16, 16], format_type: TextureFormatType::UINT };
static TF_R16G16B16A16_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [16, 16, 16, 16], format_type: TextureFormatType::SINT };
static TF_R16G16B16A16_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [16, 16, 16, 16], format_type: TextureFormatType::SFLOAT };
static TF_R32_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [32, 0, 0, 0], format_type: TextureFormatType::UINT };
static TF_R32_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [32, 0, 0, 0], format_type: TextureFormatType::SINT };
static TF_R32_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [32, 0, 0, 0], format_type: TextureFormatType::SFLOAT };
static TF_R32G32_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [32, 32, 0, 0], format_type: TextureFormatType::UINT };
static TF_R32G32_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [32, 32, 0, 0], format_type: TextureFormatType::SINT };
static TF_R32G32_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [32, 32, 0, 0], format_type: TextureFormatType::SFLOAT };
static TF_R32G32B32_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [32, 32, 32, 0], format_type: TextureFormatType::UINT };
static TF_R32G32B32_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [32, 32, 32, 0], format_type: TextureFormatType::SINT };
static TF_R32G32B32_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [32, 32, 32, 0], format_type: TextureFormatType::SFLOAT };
static TF_R32G32B32A32_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [32, 32, 32, 32], format_type: TextureFormatType::UINT };
static TF_R32G32B32A32_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [32, 32, 32, 32], format_type: TextureFormatType::SINT };
static TF_R32G32B32A32_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [32, 32, 32, 32], format_type: TextureFormatType::SFLOAT };
static TF_R64_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [64, 0, 0, 0], format_type: TextureFormatType::UINT };
static TF_R64_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [64, 0, 0, 0], format_type: TextureFormatType::SINT };
static TF_R64_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [64, 0, 0, 0], format_type: TextureFormatType::SFLOAT };
static TF_R64G64_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [64, 64, 0, 0], format_type: TextureFormatType::UINT };
static TF_R64G64_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [64, 64, 0, 0], format_type: TextureFormatType::SINT };
static TF_R64G64_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [64, 64, 0, 0], format_type: TextureFormatType::SFLOAT };
static TF_R64G64B64_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [64, 64, 64, 0], format_type: TextureFormatType::UINT };
static TF_R64G64B64_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [64, 64, 64, 0], format_type: TextureFormatType::SINT };
static TF_R64G64B64_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [64, 64, 64, 0], format_type: TextureFormatType::SFLOAT };
static TF_R64G64B64A64_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [64, 64, 64, 64], format_type: TextureFormatType::UINT };
static TF_R64G64B64A64_SINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [64, 64, 64, 64], format_type: TextureFormatType::SINT };
static TF_R64G64B64A64_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [64, 64, 64, 64], format_type: TextureFormatType::SFLOAT };
static TF_B10G11R11_UFLOAT_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::BGR, component_bits: [10, 11, 11, 0], format_type: TextureFormatType::UFLOAT };
static TF_E5B9G9R9_UFLOAT_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::EBGR, component_bits: [5, 9, 9, 9], format_type: TextureFormatType::UFLOAT };
static TF_D16_UNORM: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::D, component_bits: [16, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_X8_D24_UNORM_PACK32: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::XD, component_bits: [8, 24, 0, 0], format_type: TextureFormatType::UNORM };
static TF_D32_SFLOAT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::D, component_bits: [32, 0, 0, 0], format_type: TextureFormatType::SFLOAT };
static TF_S8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::S, component_bits: [8, 0, 0, 0], format_type: TextureFormatType::UINT };
static TF_D16_UNORM_S8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::DS, component_bits: [16, 8, 0, 0], format_type: TextureFormatType::UNORM_UINT };
static TF_D24_UNORM_S8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::DS, component_bits: [24, 8, 0, 0], format_type: TextureFormatType::UNORM_UINT };
static TF_D32_SFLOAT_S8_UINT: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::DS, component_bits: [32, 8, 0, 0], format_type: TextureFormatType::SFLOAT_UINT };
static TF_BC1_RGB_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_BC1_RGB_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_BC1_RGBA_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_BC1_RGBA_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_BC2_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_BC2_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_BC3_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_BC3_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_BC4_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_BC4_SNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SNORM };
static TF_BC5_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_BC5_SNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SNORM };
static TF_BC6H_UFLOAT_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UFLOAT };
static TF_BC6H_SFLOAT_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SFLOAT };
static TF_BC7_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_BC7_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ETC2_R8G8B8_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ETC2_R8G8B8_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGB, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ETC2_R8G8B8A1_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ETC2_R8G8B8A1_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ETC2_R8G8B8A8_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ETC2_R8G8B8A8_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_EAC_R11_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_EAC_R11_SNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::R, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SNORM };
static TF_EAC_R11G11_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_EAC_R11G11_SNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RG, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SNORM };
static TF_ASTC_4x4_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_4x4_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_5x4_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_5x4_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_5x5_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_5x5_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_6x5_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_6x5_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_6x6_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_6x6_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_8x5_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_8x5_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_8x6_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_8x6_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_8x8_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_8x8_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_10x5_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_10x5_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_10x6_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_10x6_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_10x8_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_10x8_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_10x10_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_10x10_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_12x10_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_12x10_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };
static TF_ASTC_12x12_UNORM_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::UNORM };
static TF_ASTC_12x12_SRGB_BLOCK: TextureFormatInfo = TextureFormatInfo{ component_layout: ComponentLayout::RGBA, component_bits: [0, 0, 0, 0], format_type: TextureFormatType::SRGB };

impl TextureFormat
{
    pub fn get_format_info(self) -> &'static TextureFormatInfo
    {
        match self
            {
                TextureFormat::UNDEFINED => &TF_UNDEFINED,
                TextureFormat::R4G4_UNORM_PACK8 => &TF_R4G4_UNORM_PACK8,
                TextureFormat::R4G4B4A4_UNORM_PACK16 => &TF_R4G4B4A4_UNORM_PACK16,
                TextureFormat::B4G4R4A4_UNORM_PACK16 => &TF_B4G4R4A4_UNORM_PACK16,
                TextureFormat::R5G6B5_UNORM_PACK16 => &TF_R5G6B5_UNORM_PACK16,
                TextureFormat::B5G6R5_UNORM_PACK16 => &TF_B5G6R5_UNORM_PACK16,
                TextureFormat::R5G5B5A1_UNORM_PACK16 => &TF_R5G5B5A1_UNORM_PACK16,
                TextureFormat::B5G5R5A1_UNORM_PACK16 => &TF_B5G5R5A1_UNORM_PACK16,
                TextureFormat::A1R5G5B5_UNORM_PACK16 => &TF_A1R5G5B5_UNORM_PACK16,
                TextureFormat::R8_UNORM => &TF_R8_UNORM,
                TextureFormat::R8_SNORM => &TF_R8_SNORM,
                TextureFormat::R8_USCALED => &TF_R8_USCALED,
                TextureFormat::R8_SSCALED => &TF_R8_SSCALED,
                TextureFormat::R8_UINT => &TF_R8_UINT,
                TextureFormat::R8_SINT => &TF_R8_SINT,
                TextureFormat::R8_SRGB => &TF_R8_SRGB,
                TextureFormat::R8G8_UNORM => &TF_R8G8_UNORM,
                TextureFormat::R8G8_SNORM => &TF_R8G8_SNORM,
                TextureFormat::R8G8_USCALED => &TF_R8G8_USCALED,
                TextureFormat::R8G8_SSCALED => &TF_R8G8_SSCALED,
                TextureFormat::R8G8_UINT => &TF_R8G8_UINT,
                TextureFormat::R8G8_SINT => &TF_R8G8_SINT,
                TextureFormat::R8G8_SRGB => &TF_R8G8_SRGB,
                TextureFormat::R8G8B8_UNORM => &TF_R8G8B8_UNORM,
                TextureFormat::R8G8B8_SNORM => &TF_R8G8B8_SNORM,
                TextureFormat::R8G8B8_USCALED => &TF_R8G8B8_USCALED,
                TextureFormat::R8G8B8_SSCALED => &TF_R8G8B8_SSCALED,
                TextureFormat::R8G8B8_UINT => &TF_R8G8B8_UINT,
                TextureFormat::R8G8B8_SINT => &TF_R8G8B8_SINT,
                TextureFormat::R8G8B8_SRGB => &TF_R8G8B8_SRGB,
                TextureFormat::B8G8R8_UNORM => &TF_B8G8R8_UNORM,
                TextureFormat::B8G8R8_SNORM => &TF_B8G8R8_SNORM,
                TextureFormat::B8G8R8_USCALED => &TF_B8G8R8_USCALED,
                TextureFormat::B8G8R8_SSCALED => &TF_B8G8R8_SSCALED,
                TextureFormat::B8G8R8_UINT => &TF_B8G8R8_UINT,
                TextureFormat::B8G8R8_SINT => &TF_B8G8R8_SINT,
                TextureFormat::B8G8R8_SRGB => &TF_B8G8R8_SRGB,
                TextureFormat::R8G8B8A8_UNORM => &TF_R8G8B8A8_UNORM,
                TextureFormat::R8G8B8A8_SNORM => &TF_R8G8B8A8_SNORM,
                TextureFormat::R8G8B8A8_USCALED => &TF_R8G8B8A8_USCALED,
                TextureFormat::R8G8B8A8_SSCALED => &TF_R8G8B8A8_SSCALED,
                TextureFormat::R8G8B8A8_UINT => &TF_R8G8B8A8_UINT,
                TextureFormat::R8G8B8A8_SINT => &TF_R8G8B8A8_SINT,
                TextureFormat::R8G8B8A8_SRGB => &TF_R8G8B8A8_SRGB,
                TextureFormat::B8G8R8A8_UNORM => &TF_B8G8R8A8_UNORM,
                TextureFormat::B8G8R8A8_SNORM => &TF_B8G8R8A8_SNORM,
                TextureFormat::B8G8R8A8_USCALED => &TF_B8G8R8A8_USCALED,
                TextureFormat::B8G8R8A8_SSCALED => &TF_B8G8R8A8_SSCALED,
                TextureFormat::B8G8R8A8_UINT => &TF_B8G8R8A8_UINT,
                TextureFormat::B8G8R8A8_SINT => &TF_B8G8R8A8_SINT,
                TextureFormat::B8G8R8A8_SRGB => &TF_B8G8R8A8_SRGB,
                TextureFormat::A8B8G8R8_UNORM_PACK32 => &TF_A8B8G8R8_UNORM_PACK32,
                TextureFormat::A8B8G8R8_SNORM_PACK32 => &TF_A8B8G8R8_SNORM_PACK32,
                TextureFormat::A8B8G8R8_USCALED_PACK32 => &TF_A8B8G8R8_USCALED_PACK32,
                TextureFormat::A8B8G8R8_SSCALED_PACK32 => &TF_A8B8G8R8_SSCALED_PACK32,
                TextureFormat::A8B8G8R8_UINT_PACK32 => &TF_A8B8G8R8_UINT_PACK32,
                TextureFormat::A8B8G8R8_SINT_PACK32 => &TF_A8B8G8R8_SINT_PACK32,
                TextureFormat::A8B8G8R8_SRGB_PACK32 => &TF_A8B8G8R8_SRGB_PACK32,
                TextureFormat::A2R10G10B10_UNORM_PACK32 => &TF_A2R10G10B10_UNORM_PACK32,
                TextureFormat::A2R10G10B10_SNORM_PACK32 => &TF_A2R10G10B10_SNORM_PACK32,
                TextureFormat::A2R10G10B10_USCALED_PACK32 => &TF_A2R10G10B10_USCALED_PACK32,
                TextureFormat::A2R10G10B10_SSCALED_PACK32 => &TF_A2R10G10B10_SSCALED_PACK32,
                TextureFormat::A2R10G10B10_UINT_PACK32 => &TF_A2R10G10B10_UINT_PACK32,
                TextureFormat::A2R10G10B10_SINT_PACK32 => &TF_A2R10G10B10_SINT_PACK32,
                TextureFormat::A2B10G10R10_UNORM_PACK32 => &TF_A2B10G10R10_UNORM_PACK32,
                TextureFormat::A2B10G10R10_SNORM_PACK32 => &TF_A2B10G10R10_SNORM_PACK32,
                TextureFormat::A2B10G10R10_USCALED_PACK32 => &TF_A2B10G10R10_USCALED_PACK32,
                TextureFormat::A2B10G10R10_SSCALED_PACK32 => &TF_A2B10G10R10_SSCALED_PACK32,
                TextureFormat::A2B10G10R10_UINT_PACK32 => &TF_A2B10G10R10_UINT_PACK32,
                TextureFormat::A2B10G10R10_SINT_PACK32 => &TF_A2B10G10R10_SINT_PACK32,
                TextureFormat::R16_UNORM => &TF_R16_UNORM,
                TextureFormat::R16_SNORM => &TF_R16_SNORM,
                TextureFormat::R16_USCALED => &TF_R16_USCALED,
                TextureFormat::R16_SSCALED => &TF_R16_SSCALED,
                TextureFormat::R16_UINT => &TF_R16_UINT,
                TextureFormat::R16_SINT => &TF_R16_SINT,
                TextureFormat::R16_SFLOAT => &TF_R16_SFLOAT,
                TextureFormat::R16G16_UNORM => &TF_R16G16_UNORM,
                TextureFormat::R16G16_SNORM => &TF_R16G16_SNORM,
                TextureFormat::R16G16_USCALED => &TF_R16G16_USCALED,
                TextureFormat::R16G16_SSCALED => &TF_R16G16_SSCALED,
                TextureFormat::R16G16_UINT => &TF_R16G16_UINT,
                TextureFormat::R16G16_SINT => &TF_R16G16_SINT,
                TextureFormat::R16G16_SFLOAT => &TF_R16G16_SFLOAT,
                TextureFormat::R16G16B16_UNORM => &TF_R16G16B16_UNORM,
                TextureFormat::R16G16B16_SNORM => &TF_R16G16B16_SNORM,
                TextureFormat::R16G16B16_USCALED => &TF_R16G16B16_USCALED,
                TextureFormat::R16G16B16_SSCALED => &TF_R16G16B16_SSCALED,
                TextureFormat::R16G16B16_UINT => &TF_R16G16B16_UINT,
                TextureFormat::R16G16B16_SINT => &TF_R16G16B16_SINT,
                TextureFormat::R16G16B16_SFLOAT => &TF_R16G16B16_SFLOAT,
                TextureFormat::R16G16B16A16_UNORM => &TF_R16G16B16A16_UNORM,
                TextureFormat::R16G16B16A16_SNORM => &TF_R16G16B16A16_SNORM,
                TextureFormat::R16G16B16A16_USCALED => &TF_R16G16B16A16_USCALED,
                TextureFormat::R16G16B16A16_SSCALED => &TF_R16G16B16A16_SSCALED,
                TextureFormat::R16G16B16A16_UINT => &TF_R16G16B16A16_UINT,
                TextureFormat::R16G16B16A16_SINT => &TF_R16G16B16A16_SINT,
                TextureFormat::R16G16B16A16_SFLOAT => &TF_R16G16B16A16_SFLOAT,
                TextureFormat::R32_UINT => &TF_R32_UINT,
                TextureFormat::R32_SINT => &TF_R32_SINT,
                TextureFormat::R32_SFLOAT => &TF_R32_SFLOAT,
                TextureFormat::R32G32_UINT => &TF_R32G32_UINT,
                TextureFormat::R32G32_SINT => &TF_R32G32_SINT,
                TextureFormat::R32G32_SFLOAT => &TF_R32G32_SFLOAT,
                TextureFormat::R32G32B32_UINT => &TF_R32G32B32_UINT,
                TextureFormat::R32G32B32_SINT => &TF_R32G32B32_SINT,
                TextureFormat::R32G32B32_SFLOAT => &TF_R32G32B32_SFLOAT,
                TextureFormat::R32G32B32A32_UINT => &TF_R32G32B32A32_UINT,
                TextureFormat::R32G32B32A32_SINT => &TF_R32G32B32A32_SINT,
                TextureFormat::R32G32B32A32_SFLOAT => &TF_R32G32B32A32_SFLOAT,
                TextureFormat::R64_UINT => &TF_R64_UINT,
                TextureFormat::R64_SINT => &TF_R64_SINT,
                TextureFormat::R64_SFLOAT => &TF_R64_SFLOAT,
                TextureFormat::R64G64_UINT => &TF_R64G64_UINT,
                TextureFormat::R64G64_SINT => &TF_R64G64_SINT,
                TextureFormat::R64G64_SFLOAT => &TF_R64G64_SFLOAT,
                TextureFormat::R64G64B64_UINT => &TF_R64G64B64_UINT,
                TextureFormat::R64G64B64_SINT => &TF_R64G64B64_SINT,
                TextureFormat::R64G64B64_SFLOAT => &TF_R64G64B64_SFLOAT,
                TextureFormat::R64G64B64A64_UINT => &TF_R64G64B64A64_UINT,
                TextureFormat::R64G64B64A64_SINT => &TF_R64G64B64A64_SINT,
                TextureFormat::R64G64B64A64_SFLOAT => &TF_R64G64B64A64_SFLOAT,
                TextureFormat::B10G11R11_UFLOAT_PACK32 => &TF_B10G11R11_UFLOAT_PACK32,
                TextureFormat::E5B9G9R9_UFLOAT_PACK32 => &TF_E5B9G9R9_UFLOAT_PACK32,
                TextureFormat::D16_UNORM => &TF_D16_UNORM,
                TextureFormat::X8_D24_UNORM_PACK32 => &TF_X8_D24_UNORM_PACK32,
                TextureFormat::D32_SFLOAT => &TF_D32_SFLOAT,
                TextureFormat::S8_UINT => &TF_S8_UINT,
                TextureFormat::D16_UNORM_S8_UINT => &TF_D16_UNORM_S8_UINT,
                TextureFormat::D24_UNORM_S8_UINT => &TF_D24_UNORM_S8_UINT,
                TextureFormat::D32_SFLOAT_S8_UINT => &TF_D32_SFLOAT_S8_UINT,
                TextureFormat::BC1_RGB_UNORM_BLOCK => &TF_BC1_RGB_UNORM_BLOCK,
                TextureFormat::BC1_RGB_SRGB_BLOCK => &TF_BC1_RGB_SRGB_BLOCK,
                TextureFormat::BC1_RGBA_UNORM_BLOCK => &TF_BC1_RGBA_UNORM_BLOCK,
                TextureFormat::BC1_RGBA_SRGB_BLOCK => &TF_BC1_RGBA_SRGB_BLOCK,
                TextureFormat::BC2_UNORM_BLOCK => &TF_BC2_UNORM_BLOCK,
                TextureFormat::BC2_SRGB_BLOCK => &TF_BC2_SRGB_BLOCK,
                TextureFormat::BC3_UNORM_BLOCK => &TF_BC3_UNORM_BLOCK,
                TextureFormat::BC3_SRGB_BLOCK => &TF_BC3_SRGB_BLOCK,
                TextureFormat::BC4_UNORM_BLOCK => &TF_BC4_UNORM_BLOCK,
                TextureFormat::BC4_SNORM_BLOCK => &TF_BC4_SNORM_BLOCK,
                TextureFormat::BC5_UNORM_BLOCK => &TF_BC5_UNORM_BLOCK,
                TextureFormat::BC5_SNORM_BLOCK => &TF_BC5_SNORM_BLOCK,
                TextureFormat::BC6H_UFLOAT_BLOCK => &TF_BC6H_UFLOAT_BLOCK,
                TextureFormat::BC6H_SFLOAT_BLOCK => &TF_BC6H_SFLOAT_BLOCK,
                TextureFormat::BC7_UNORM_BLOCK => &TF_BC7_UNORM_BLOCK,
                TextureFormat::BC7_SRGB_BLOCK => &TF_BC7_SRGB_BLOCK,
                TextureFormat::ETC2_R8G8B8_UNORM_BLOCK => &TF_ETC2_R8G8B8_UNORM_BLOCK,
                TextureFormat::ETC2_R8G8B8_SRGB_BLOCK => &TF_ETC2_R8G8B8_SRGB_BLOCK,
                TextureFormat::ETC2_R8G8B8A1_UNORM_BLOCK => &TF_ETC2_R8G8B8A1_UNORM_BLOCK,
                TextureFormat::ETC2_R8G8B8A1_SRGB_BLOCK => &TF_ETC2_R8G8B8A1_SRGB_BLOCK,
                TextureFormat::ETC2_R8G8B8A8_UNORM_BLOCK => &TF_ETC2_R8G8B8A8_UNORM_BLOCK,
                TextureFormat::ETC2_R8G8B8A8_SRGB_BLOCK => &TF_ETC2_R8G8B8A8_SRGB_BLOCK,
                TextureFormat::EAC_R11_UNORM_BLOCK => &TF_EAC_R11_UNORM_BLOCK,
                TextureFormat::EAC_R11_SNORM_BLOCK => &TF_EAC_R11_SNORM_BLOCK,
                TextureFormat::EAC_R11G11_UNORM_BLOCK => &TF_EAC_R11G11_UNORM_BLOCK,
                TextureFormat::EAC_R11G11_SNORM_BLOCK => &TF_EAC_R11G11_SNORM_BLOCK,
                TextureFormat::ASTC_4x4_UNORM_BLOCK => &TF_ASTC_4x4_UNORM_BLOCK,
                TextureFormat::ASTC_4x4_SRGB_BLOCK => &TF_ASTC_4x4_SRGB_BLOCK,
                TextureFormat::ASTC_5x4_UNORM_BLOCK => &TF_ASTC_5x4_UNORM_BLOCK,
                TextureFormat::ASTC_5x4_SRGB_BLOCK => &TF_ASTC_5x4_SRGB_BLOCK,
                TextureFormat::ASTC_5x5_UNORM_BLOCK => &TF_ASTC_5x5_UNORM_BLOCK,
                TextureFormat::ASTC_5x5_SRGB_BLOCK => &TF_ASTC_5x5_SRGB_BLOCK,
                TextureFormat::ASTC_6x5_UNORM_BLOCK => &TF_ASTC_6x5_UNORM_BLOCK,
                TextureFormat::ASTC_6x5_SRGB_BLOCK => &TF_ASTC_6x5_SRGB_BLOCK,
                TextureFormat::ASTC_6x6_UNORM_BLOCK => &TF_ASTC_6x6_UNORM_BLOCK,
                TextureFormat::ASTC_6x6_SRGB_BLOCK => &TF_ASTC_6x6_SRGB_BLOCK,
                TextureFormat::ASTC_8x5_UNORM_BLOCK => &TF_ASTC_8x5_UNORM_BLOCK,
                TextureFormat::ASTC_8x5_SRGB_BLOCK => &TF_ASTC_8x5_SRGB_BLOCK,
                TextureFormat::ASTC_8x6_UNORM_BLOCK => &TF_ASTC_8x6_UNORM_BLOCK,
                TextureFormat::ASTC_8x6_SRGB_BLOCK => &TF_ASTC_8x6_SRGB_BLOCK,
                TextureFormat::ASTC_8x8_UNORM_BLOCK => &TF_ASTC_8x8_UNORM_BLOCK,
                TextureFormat::ASTC_8x8_SRGB_BLOCK => &TF_ASTC_8x8_SRGB_BLOCK,
                TextureFormat::ASTC_10x5_UNORM_BLOCK => &TF_ASTC_10x5_UNORM_BLOCK,
                TextureFormat::ASTC_10x5_SRGB_BLOCK => &TF_ASTC_10x5_SRGB_BLOCK,
                TextureFormat::ASTC_10x6_UNORM_BLOCK => &TF_ASTC_10x6_UNORM_BLOCK,
                TextureFormat::ASTC_10x6_SRGB_BLOCK => &TF_ASTC_10x6_SRGB_BLOCK,
                TextureFormat::ASTC_10x8_UNORM_BLOCK => &TF_ASTC_10x8_UNORM_BLOCK,
                TextureFormat::ASTC_10x8_SRGB_BLOCK => &TF_ASTC_10x8_SRGB_BLOCK,
                TextureFormat::ASTC_10x10_UNORM_BLOCK => &TF_ASTC_10x10_UNORM_BLOCK,
                TextureFormat::ASTC_10x10_SRGB_BLOCK => &TF_ASTC_10x10_SRGB_BLOCK,
                TextureFormat::ASTC_12x10_UNORM_BLOCK => &TF_ASTC_12x10_UNORM_BLOCK,
                TextureFormat::ASTC_12x10_SRGB_BLOCK => &TF_ASTC_12x10_SRGB_BLOCK,
                TextureFormat::ASTC_12x12_UNORM_BLOCK => &TF_ASTC_12x12_UNORM_BLOCK,
                TextureFormat::ASTC_12x12_SRGB_BLOCK => &TF_ASTC_12x12_SRGB_BLOCK,
            }
    }
}

pub struct GlFormatInfo
{
    pub internal_fmt: GLenum,
    pub upload_components: GLenum,         //< Matching external format for uploads/reads (so that OpenGL does not have to do any conversion)
    pub upload_ty: GLenum,        //< Matching element type for uploads/reads
}

static GLF_R8_UNORM: GlFormatInfo = GlFormatInfo { internal_fmt: gl::R8, upload_components: gl::RED, upload_ty: gl::UNSIGNED_BYTE };
static GLF_R8_SNORM: GlFormatInfo = GlFormatInfo { internal_fmt: gl::R8_SNORM, upload_components: gl::RED, upload_ty: gl::BYTE };
static GLF_R8_UINT: GlFormatInfo = GlFormatInfo { internal_fmt: gl::R8UI, upload_components: gl::RED, upload_ty: gl::UNSIGNED_BYTE };
static GLF_R8_SINT: GlFormatInfo = GlFormatInfo { internal_fmt: gl::R8I, upload_components: gl::RED, upload_ty: gl::BYTE };
static GLF_R16G16_SFLOAT: GlFormatInfo = GlFormatInfo { internal_fmt: gl::RG16F, upload_components: gl::RG, upload_ty: gl::FLOAT };    // XXX no half-float for upload!
static GLF_R16G16B16A16_SFLOAT: GlFormatInfo = GlFormatInfo { internal_fmt: gl::RGBA16F, upload_components: gl::RGBA, upload_ty: gl::FLOAT };    // XXX no half-float for upload!
static GLF_R32G32_SFLOAT: GlFormatInfo = GlFormatInfo { internal_fmt: gl::RG32F, upload_components: gl::RG, upload_ty: gl::FLOAT };
static GLF_R32G32B32A32_SFLOAT: GlFormatInfo = GlFormatInfo { internal_fmt: gl::RGBA32F, upload_components: gl::RGBA, upload_ty: gl::FLOAT };
static GLF_R8G8B8A8_UNORM: GlFormatInfo = GlFormatInfo { internal_fmt: gl::RGBA8, upload_components: gl::RGBA, upload_ty: gl::UNSIGNED_BYTE };
static GLF_R8G8B8A8_SNORM: GlFormatInfo = GlFormatInfo { internal_fmt: gl::RGBA8_SNORM, upload_components: gl::RGBA, upload_ty: gl::BYTE };
static GLF_R8G8B8A8_UINT: GlFormatInfo = GlFormatInfo { internal_fmt: gl::RGBA8UI, upload_components: gl::RGBA, upload_ty: gl::UNSIGNED_BYTE };
static GLF_R8G8B8A8_SINT: GlFormatInfo = GlFormatInfo { internal_fmt: gl::RGBA8I, upload_components: gl::RGBA, upload_ty: gl::BYTE };
static GLF_R8G8B8_SRGB: GlFormatInfo = GlFormatInfo { internal_fmt: gl::SRGB8, upload_components: gl::RGB, upload_ty: gl::UNSIGNED_BYTE };
static GLF_R8G8B8A8_SRGB: GlFormatInfo = GlFormatInfo { internal_fmt: gl::SRGB8_ALPHA8, upload_components: gl::RGBA, upload_ty: gl::UNSIGNED_BYTE };

impl GlFormatInfo {
    pub fn from_texture_format(fmt: TextureFormat) -> &'static GlFormatInfo {
        match fmt {
            TextureFormat::R8_UNORM => &GLF_R8_UNORM,
            TextureFormat::R8_SNORM => &GLF_R8_SNORM,
            TextureFormat::R8_UINT => &GLF_R8_UINT,
            TextureFormat::R8_SINT => &GLF_R8_SINT,
            TextureFormat::R16G16_SFLOAT => &GLF_R16G16_SFLOAT,
            TextureFormat::R16G16B16A16_SFLOAT => &GLF_R16G16B16A16_SFLOAT,
            TextureFormat::R32G32_SFLOAT => &GLF_R32G32_SFLOAT,
            TextureFormat::R32G32B32A32_SFLOAT => &GLF_R32G32B32A32_SFLOAT,
            TextureFormat::R8G8B8A8_UNORM => &GLF_R8G8B8A8_UNORM,
            TextureFormat::R8G8B8A8_SNORM => &GLF_R8G8B8A8_SNORM,
            TextureFormat::R8G8B8A8_UINT => &GLF_R8G8B8A8_UINT,
            TextureFormat::R8G8B8A8_SINT => &GLF_R8G8B8A8_SINT,
            TextureFormat::R8G8B8_SRGB => &GLF_R8G8B8_SRGB,
            TextureFormat::R8G8B8A8_SRGB => &GLF_R8G8B8A8_SRGB,
            _ => panic!("Unsupported image format")
        }
    }
}
