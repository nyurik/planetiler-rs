/// 32 bit encoding of the tile X,Y for zooms 0-15.
/// First two bits: 11 = z15, 10 = z14, 00 = z0..13
/// For z0..13, next 4 bits is the zoom value,
/// followed by 2 13-bit values for x and y.
///
/// This table shows maximum possible value for each
///  z00:  00:0000:·············:············· ( 0 bits for x,y)
///  z01:  00:0001:············1:············1 ( 2 bits for x,y)
///  z02:  00:0010:···········11:···········11 ( 4 bits for x,y)
///  z03:  00:0011:··········111:··········111 ( 6 bits for x,y)
///  z04:  00:0100:·········1111:·········1111 ( 8 bits for x,y)
///  z05:  00:0101:········11111:········11111 (10 bits for x,y)
///  z06:  00:0110:·······111111:·······111111 (12 bits for x,y)
///  z07:  00:0111:······1111111:······1111111 (14 bits for x,y)
///  z08:  00:1000:·····11111111:·····11111111 (16 bits for x,y)
///  z09:  00:1001:····111111111:····111111111 (18 bits for x,y)
///  z10:  00:1010:···1111111111:···1111111111 (20 bits for x,y)
///  z11:  00:1011:··11111111111:··11111111111 (22 bits for x,y)
///  z12:  00:1100:·111111111111:·111111111111 (24 bits for x,y)
///  z13:  00:1101:1111111111111:1111111111111 (26 bits for x,y)
///  z14:  10::·11111111111111:·11111111111111 (28 bits for x,y)
///  z15:  11;:111111111111111:111111111111111 (30 bits for x,y)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PackedTileID(u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TileID {
    pub x: u32,
    pub y: u32,
    pub zoom: u32,
}

impl PackedTileID {
    #[allow(dead_code)]
    pub fn new(id: TileID) -> Self {
        // todo: validation
        match id.zoom {
            0..=13 => Self((id.zoom << 26) | (id.x << 13) | id.y),
            14..=15 => Self(((id.zoom - 12) << 30) | (id.x << 15) | id.y),
            _ => {
                panic!()
            }
        }
    }

    #[allow(dead_code)]
    pub fn decode(&self) -> TileID {
        let zoom = (self.0 & 0b1111_1100_0000_0000_0000_0000_0000_0000) >> 26;
        if zoom <= 13 {
            TileID {
                x: (self.0 & 0b0000_0011_1111_1111_1110_0000_0000_0000) >> 13,
                y: (self.0 & 0b0000_0000_0000_0000_0001_1111_1111_1111),
                zoom,
            }
        } else {
            TileID {
                x: (self.0 & 0b0011_1111_1111_1111_1000_0000_0000_0000) >> 15,
                y: (self.0 & 0b0000_0000_0000_0000_0111_1111_1111_1111),
                zoom: ((zoom & 0b110000) >> 4) + 12, // either 0b10 -> 14, or 0b11 -> 15
            }
        }
    }
}
