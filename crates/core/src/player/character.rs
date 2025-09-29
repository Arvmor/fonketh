use crate::movements::{Motion, Position};
use crate::prelude::{Color, Frame};
use crate::utils::Identifier;

/// Represents a pixelated character sprite
#[derive(Debug, Clone)]
pub struct PixelatedCharacter {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec<Color>>,
}

impl PixelatedCharacter {
    /// Create a small grayscale farmer character
    pub fn new_farmer() -> Self {
        // Create a 8x8 pixel grayscale character
        let width = 8;
        let height = 8;
        let mut pixels = vec![vec![Color::Reset; width]; height];

        // Grayscale colors
        let skin = Color::Rgb(200, 200, 200); // Light gray skin
        let hair = Color::Rgb(100, 100, 100); // Dark gray hair
        let shirt = Color::Rgb(150, 150, 150); // Medium gray shirt
        let pants = Color::Rgb(80, 80, 80); // Dark gray pants
        let black = Color::Rgb(0, 0, 0); // Black for eyes

        // Simple 8x8 grayscale character
        // Head (rows 0-2)
        pixels[0][2] = hair;
        pixels[0][3] = hair;
        pixels[0][4] = hair;
        pixels[0][5] = hair;
        pixels[1][1] = hair;
        pixels[1][2] = hair;
        pixels[1][3] = hair;
        pixels[1][4] = hair;
        pixels[1][5] = hair;
        pixels[1][6] = hair;
        pixels[2][2] = skin;
        pixels[2][3] = skin;
        pixels[2][4] = skin;
        pixels[2][5] = skin;

        // Eyes
        pixels[2][3] = black;
        pixels[2][4] = black;

        // Body (rows 3-5)
        pixels[3][2] = shirt;
        pixels[3][3] = shirt;
        pixels[3][4] = shirt;
        pixels[3][5] = shirt;
        pixels[4][2] = shirt;
        pixels[4][3] = shirt;
        pixels[4][4] = shirt;
        pixels[4][5] = shirt;
        pixels[5][2] = shirt;
        pixels[5][3] = shirt;
        pixels[5][4] = shirt;
        pixels[5][5] = shirt;

        // Arms
        pixels[4][1] = skin;
        pixels[4][6] = skin;

        // Legs (rows 6-7)
        pixels[6][2] = pants;
        pixels[6][3] = pants;
        pixels[6][4] = pants;
        pixels[6][5] = pants;
        pixels[7][2] = pants;
        pixels[7][3] = pants;
        pixels[7][4] = pants;
        pixels[7][5] = pants;

        Self {
            width,
            height,
            pixels,
        }
    }

    /// Create a small grayscale villager character
    pub fn new_villager() -> Self {
        // Create a 8x8 pixel grayscale villager
        let width = 8;
        let height = 8;
        let mut pixels = vec![vec![Color::Reset; width]; height];

        // Grayscale colors
        let skin = Color::Rgb(200, 200, 200); // Light gray skin
        let hair = Color::Rgb(120, 120, 120); // Medium gray hair
        let dress = Color::Rgb(100, 100, 100); // Dark gray dress
        let black = Color::Rgb(0, 0, 0); // Black for eyes

        // Simple 8x8 grayscale villager
        // Head (rows 0-2)
        pixels[0][2] = hair;
        pixels[0][3] = hair;
        pixels[0][4] = hair;
        pixels[0][5] = hair;
        pixels[1][1] = hair;
        pixels[1][2] = hair;
        pixels[1][3] = hair;
        pixels[1][4] = hair;
        pixels[1][5] = hair;
        pixels[1][6] = hair;
        pixels[2][2] = skin;
        pixels[2][3] = skin;
        pixels[2][4] = skin;
        pixels[2][5] = skin;

        // Eyes
        pixels[2][3] = black;
        pixels[2][4] = black;

        // Body/Dress (rows 3-7)
        pixels[3][2] = dress;
        pixels[3][3] = dress;
        pixels[3][4] = dress;
        pixels[3][5] = dress;
        pixels[4][2] = dress;
        pixels[4][3] = dress;
        pixels[4][4] = dress;
        pixels[4][5] = dress;
        pixels[5][2] = dress;
        pixels[5][3] = dress;
        pixels[5][4] = dress;
        pixels[5][5] = dress;
        pixels[6][2] = dress;
        pixels[6][3] = dress;
        pixels[6][4] = dress;
        pixels[6][5] = dress;
        pixels[7][2] = dress;
        pixels[7][3] = dress;
        pixels[7][4] = dress;
        pixels[7][5] = dress;

        // Arms
        pixels[4][1] = skin;
        pixels[4][6] = skin;

        Self {
            width,
            height,
            pixels,
        }
    }

    /// Create a small grayscale merchant character
    pub fn new_merchant() -> Self {
        // Create a 8x8 pixel grayscale merchant
        let width = 8;
        let height = 8;
        let mut pixels = vec![vec![Color::Reset; width]; height];

        // Grayscale colors
        let skin = Color::Rgb(200, 200, 200); // Light gray skin
        let hair = Color::Rgb(80, 80, 80); // Dark gray hair
        let vest = Color::Rgb(60, 60, 60); // Very dark gray vest
        let shirt = Color::Rgb(180, 180, 180); // Light gray shirt
        let _pants = Color::Rgb(40, 40, 40); // Very dark gray pants
        let black = Color::Rgb(0, 0, 0); // Black for eyes

        // Simple 8x8 grayscale merchant
        // Head (rows 0-2)
        pixels[0][2] = hair;
        pixels[0][3] = hair;
        pixels[0][4] = hair;
        pixels[0][5] = hair;
        pixels[1][1] = hair;
        pixels[1][2] = hair;
        pixels[1][3] = hair;
        pixels[1][4] = hair;
        pixels[1][5] = hair;
        pixels[1][6] = hair;
        pixels[2][2] = skin;
        pixels[2][3] = skin;
        pixels[2][4] = skin;
        pixels[2][5] = skin;

        // Eyes
        pixels[2][3] = black;
        pixels[2][4] = black;

        // Mustache
        pixels[2][3] = hair;
        pixels[2][4] = hair;

        // Body/Vest (rows 3-7)
        pixels[3][2] = vest;
        pixels[3][3] = vest;
        pixels[3][4] = vest;
        pixels[3][5] = vest;
        pixels[4][2] = vest;
        pixels[4][3] = vest;
        pixels[4][4] = vest;
        pixels[4][5] = vest;
        pixels[5][2] = vest;
        pixels[5][3] = vest;
        pixels[5][4] = vest;
        pixels[5][5] = vest;
        pixels[6][2] = vest;
        pixels[6][3] = vest;
        pixels[6][4] = vest;
        pixels[6][5] = vest;
        pixels[7][2] = vest;
        pixels[7][3] = vest;
        pixels[7][4] = vest;
        pixels[7][5] = vest;

        // White shirt underneath
        pixels[4][3] = shirt;
        pixels[4][4] = shirt;
        pixels[5][3] = shirt;
        pixels[5][4] = shirt;

        // Arms
        pixels[4][1] = skin;
        pixels[4][6] = skin;

        Self {
            width,
            height,
            pixels,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Character<N, B> {
    pub name: N,
    pub balance: B,
    pub position: Position,
    pub sprite: PixelatedCharacter,
}

impl<N, B> Character<N, B> {
    pub fn new(name: N, balance: B) -> Self {
        let position = Position::default();
        let sprite = PixelatedCharacter::new_farmer();

        Self {
            name,
            balance,
            position,
            sprite,
        }
    }

    pub fn new_with_sprite(name: N, balance: B, sprite: PixelatedCharacter) -> Self {
        let position = Position::default();

        Self {
            name,
            balance,
            position,
            sprite,
        }
    }

    pub fn name(&self) -> &N {
        &self.name
    }

    pub fn balance(&self) -> &B {
        &self.balance
    }
}

impl<N, B> Motion for Character<N, B> {
    fn r#move(&self, _frame: &mut Frame) {}
}

impl<N: Clone, B> Identifier for Character<N, B> {
    type Id = N;

    fn identifier(&self) -> Self::Id {
        self.name.clone()
    }
}
