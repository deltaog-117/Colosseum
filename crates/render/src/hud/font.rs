//! The built-in bitmap font: a table of 5x7 glyphs packed into an atlas of
//! 8x8 cells, so the HUD needs no asset files. A row reads left to right in the
//! high five bits, grouped as `0bLLLL_RDDD` with the last three always zero.

/// Edge of one atlas cell, in texels.
pub const CELL: u32 = 8;
/// Cells per atlas row.
pub const COLUMNS: u32 = 8;
/// Atlas width in texels.
pub const ATLAS_WIDTH: u32 = CELL * COLUMNS;
/// Atlas height in texels.
pub const ATLAS_HEIGHT: u32 = CELL * 6;
/// The cell that is solid, for rectangles.
pub const SOLID_CELL: u32 = 0;

/// Each row holds five pixels in the high bits, left pixel first.
type Rows = [u8; 7];

/// Drawn for a character the font does not have.
const UNKNOWN: char = '?';

/// Cell `n + 1` holds entry `n`; cell 0 is the solid block.
const GLYPHS: [(char, Rows); 45] = [
    (' ', [0; 7]),
    (
        '0',
        [
            0b0111_0000,
            0b1000_1000,
            0b1001_1000,
            0b1010_1000,
            0b1100_1000,
            0b1000_1000,
            0b0111_0000,
        ],
    ),
    (
        '1',
        [
            0b0010_0000,
            0b0110_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0111_0000,
        ],
    ),
    (
        '2',
        [
            0b0111_0000,
            0b1000_1000,
            0b0000_1000,
            0b0001_0000,
            0b0010_0000,
            0b0100_0000,
            0b1111_1000,
        ],
    ),
    (
        '3',
        [
            0b1111_0000,
            0b0000_1000,
            0b0000_1000,
            0b0111_0000,
            0b0000_1000,
            0b0000_1000,
            0b1111_0000,
        ],
    ),
    (
        '4',
        [
            0b0001_0000,
            0b0011_0000,
            0b0101_0000,
            0b1001_0000,
            0b1111_1000,
            0b0001_0000,
            0b0001_0000,
        ],
    ),
    (
        '5',
        [
            0b1111_1000,
            0b1000_0000,
            0b1111_0000,
            0b0000_1000,
            0b0000_1000,
            0b1000_1000,
            0b0111_0000,
        ],
    ),
    (
        '6',
        [
            0b0011_0000,
            0b0100_0000,
            0b1000_0000,
            0b1111_0000,
            0b1000_1000,
            0b1000_1000,
            0b0111_0000,
        ],
    ),
    (
        '7',
        [
            0b1111_1000,
            0b0000_1000,
            0b0001_0000,
            0b0010_0000,
            0b0100_0000,
            0b0100_0000,
            0b0100_0000,
        ],
    ),
    (
        '8',
        [
            0b0111_0000,
            0b1000_1000,
            0b1000_1000,
            0b0111_0000,
            0b1000_1000,
            0b1000_1000,
            0b0111_0000,
        ],
    ),
    (
        '9',
        [
            0b0111_0000,
            0b1000_1000,
            0b1000_1000,
            0b0111_1000,
            0b0000_1000,
            0b0001_0000,
            0b0110_0000,
        ],
    ),
    (
        'A',
        [
            0b0111_0000,
            0b1000_1000,
            0b1000_1000,
            0b1111_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
        ],
    ),
    (
        'B',
        [
            0b1111_0000,
            0b1000_1000,
            0b1000_1000,
            0b1111_0000,
            0b1000_1000,
            0b1000_1000,
            0b1111_0000,
        ],
    ),
    (
        'C',
        [
            0b0111_0000,
            0b1000_1000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_1000,
            0b0111_0000,
        ],
    ),
    (
        'D',
        [
            0b1111_0000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1111_0000,
        ],
    ),
    (
        'E',
        [
            0b1111_1000,
            0b1000_0000,
            0b1000_0000,
            0b1111_0000,
            0b1000_0000,
            0b1000_0000,
            0b1111_1000,
        ],
    ),
    (
        'F',
        [
            0b1111_1000,
            0b1000_0000,
            0b1000_0000,
            0b1111_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
        ],
    ),
    (
        'G',
        [
            0b0111_0000,
            0b1000_1000,
            0b1000_0000,
            0b1011_1000,
            0b1000_1000,
            0b1000_1000,
            0b0111_1000,
        ],
    ),
    (
        'H',
        [
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1111_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
        ],
    ),
    (
        'I',
        [
            0b0111_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0111_0000,
        ],
    ),
    (
        'J',
        [
            0b0011_1000,
            0b0001_0000,
            0b0001_0000,
            0b0001_0000,
            0b0001_0000,
            0b1001_0000,
            0b0110_0000,
        ],
    ),
    (
        'K',
        [
            0b1000_1000,
            0b1001_0000,
            0b1010_0000,
            0b1100_0000,
            0b1010_0000,
            0b1001_0000,
            0b1000_1000,
        ],
    ),
    (
        'L',
        [
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1111_1000,
        ],
    ),
    (
        'M',
        [
            0b1000_1000,
            0b1101_1000,
            0b1010_1000,
            0b1010_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
        ],
    ),
    (
        'N',
        [
            0b1000_1000,
            0b1100_1000,
            0b1010_1000,
            0b1001_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
        ],
    ),
    (
        'O',
        [
            0b0111_0000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b0111_0000,
        ],
    ),
    (
        'P',
        [
            0b1111_0000,
            0b1000_1000,
            0b1000_1000,
            0b1111_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
        ],
    ),
    (
        'Q',
        [
            0b0111_0000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1010_1000,
            0b1001_0000,
            0b0110_1000,
        ],
    ),
    (
        'R',
        [
            0b1111_0000,
            0b1000_1000,
            0b1000_1000,
            0b1111_0000,
            0b1010_0000,
            0b1001_0000,
            0b1000_1000,
        ],
    ),
    (
        'S',
        [
            0b0111_1000,
            0b1000_0000,
            0b1000_0000,
            0b0111_0000,
            0b0000_1000,
            0b0000_1000,
            0b1111_0000,
        ],
    ),
    (
        'T',
        [
            0b1111_1000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
        ],
    ),
    (
        'U',
        [
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b0111_0000,
        ],
    ),
    (
        'V',
        [
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b0101_0000,
            0b0010_0000,
        ],
    ),
    (
        'W',
        [
            0b1000_1000,
            0b1000_1000,
            0b1000_1000,
            0b1010_1000,
            0b1010_1000,
            0b1101_1000,
            0b1000_1000,
        ],
    ),
    (
        'X',
        [
            0b1000_1000,
            0b1000_1000,
            0b0101_0000,
            0b0010_0000,
            0b0101_0000,
            0b1000_1000,
            0b1000_1000,
        ],
    ),
    (
        'Y',
        [
            0b1000_1000,
            0b1000_1000,
            0b0101_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
        ],
    ),
    (
        'Z',
        [
            0b1111_1000,
            0b0000_1000,
            0b0001_0000,
            0b0010_0000,
            0b0100_0000,
            0b1000_0000,
            0b1111_1000,
        ],
    ),
    (
        ':',
        [
            0b0000_0000,
            0b0010_0000,
            0b0010_0000,
            0b0000_0000,
            0b0010_0000,
            0b0010_0000,
            0b0000_0000,
        ],
    ),
    (
        '-',
        [
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b1111_1000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
        ],
    ),
    (
        '/',
        [
            0b0000_1000,
            0b0000_1000,
            0b0001_0000,
            0b0010_0000,
            0b0100_0000,
            0b1000_0000,
            0b1000_0000,
        ],
    ),
    (
        '.',
        [
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0110_0000,
            0b0110_0000,
        ],
    ),
    (
        '+',
        [
            0b0000_0000,
            0b0010_0000,
            0b0010_0000,
            0b1111_1000,
            0b0010_0000,
            0b0010_0000,
            0b0000_0000,
        ],
    ),
    (
        '!',
        [
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0010_0000,
            0b0000_0000,
            0b0010_0000,
        ],
    ),
    (
        '%',
        [
            0b1100_1000,
            0b1101_0000,
            0b0001_0000,
            0b0010_0000,
            0b0100_0000,
            0b0101_1000,
            0b1001_1000,
        ],
    ),
    (
        '?',
        [
            0b0111_0000,
            0b1000_1000,
            0b0000_1000,
            0b0001_0000,
            0b0010_0000,
            0b0000_0000,
            0b0010_0000,
        ],
    ),
];

/// The atlas cell that draws `ch`. Lowercase letters share the uppercase
/// glyphs, and a character the font lacks gets the cell of `?`, so any text
/// can be drawn.
pub fn cell_for(ch: char) -> u32 {
    let ch = ch.to_ascii_uppercase();
    let position = GLYPHS
        .iter()
        .position(|(glyph, _)| *glyph == ch)
        .or_else(|| GLYPHS.iter().position(|(glyph, _)| *glyph == UNKNOWN));
    // `UNKNOWN` is in the table, so the fallback always finds it; the solid
    // cell is the harmless answer if the table is ever edited wrongly.
    position.map_or(SOLID_CELL, |index| index as u32 + 1)
}

/// The atlas as one byte per texel, row by row: 255 where a glyph draws,
/// 0 elsewhere. Cell 0 is solid.
pub fn pixels() -> Vec<u8> {
    let mut atlas = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT) as usize];
    let mut paint = |cell: u32, row: u32, bits: u8| {
        let (cell_x, cell_y) = ((cell % COLUMNS) * CELL, (cell / COLUMNS) * CELL);
        for column in 0..CELL {
            if bits & (0x80 >> column) != 0 {
                let index = (cell_y + row) * ATLAS_WIDTH + cell_x + column;
                atlas[index as usize] = 255;
            }
        }
    };
    for row in 0..CELL {
        paint(SOLID_CELL, row, 0xFF);
    }
    for (index, (_, rows)) in GLYPHS.iter().enumerate() {
        for (row, bits) in rows.iter().enumerate() {
            paint(index as u32 + 1, row as u32, *bits);
        }
    }
    atlas
}

/// The corners of a cell in texture coordinates: `[u0, v0, u1, v1]`.
pub fn cell_uv(cell: u32) -> [f32; 4] {
    let (width, height) = (ATLAS_WIDTH as f32, ATLAS_HEIGHT as f32);
    let (x, y) = (
        (cell % COLUMNS * CELL) as f32,
        (cell / COLUMNS * CELL) as f32,
    );
    [
        x / width,
        y / height,
        (x + CELL as f32) / width,
        (y + CELL as f32) / height,
    ]
}
