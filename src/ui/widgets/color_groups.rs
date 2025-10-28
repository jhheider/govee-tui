/// Convert CamelCase color name to spaced English name
pub fn to_spaced_name(camel: &str) -> String {
    let mut result = String::with_capacity(camel.len() + 5);
    let mut chars = camel.chars().peekable();

    if let Some(first) = chars.next() {
        result.push(first);
    }

    for ch in chars {
        if ch.is_uppercase() {
            result.push(' ');
        }
        result.push(ch);
    }

    result
}

/// Color group with emoji swatch
#[derive(Debug, Clone)]
pub struct ColorGroup {
    pub name: &'static str,
    pub emoji: &'static str,
    pub colors: Vec<(&'static str, [u8; 3])>,
}

/// Get all color groups organized by family
pub fn get_color_groups() -> Vec<ColorGroup> {
    vec![
        ColorGroup {
            name: "Reds",
            emoji: "🔴",
            colors: vec![
                ("IndianRed", [205, 92, 92]),
                ("LightCoral", [240, 128, 128]),
                ("Salmon", [250, 128, 114]),
                ("DarkSalmon", [233, 150, 122]),
                ("LightSalmon", [255, 160, 122]),
                ("Crimson", [220, 20, 60]),
                ("Red", [255, 0, 0]),
                ("FireBrick", [178, 34, 34]),
                ("DarkRed", [139, 0, 0]),
            ],
        },
        ColorGroup {
            name: "Oranges",
            emoji: "🟠",
            colors: vec![
                ("Coral", [255, 127, 80]),
                ("Tomato", [255, 99, 71]),
                ("OrangeRed", [255, 69, 0]),
                ("DarkOrange", [255, 140, 0]),
                ("Orange", [255, 165, 0]),
            ],
        },
        ColorGroup {
            name: "Yellows",
            emoji: "🟡",
            colors: vec![
                ("Gold", [255, 215, 0]),
                ("Yellow", [255, 255, 0]),
                ("LightYellow", [255, 255, 224]),
                ("LemonChiffon", [255, 250, 205]),
                ("Moccasin", [255, 228, 181]),
                ("PeachPuff", [255, 218, 185]),
                ("Khaki", [240, 230, 140]),
            ],
        },
        ColorGroup {
            name: "Greens",
            emoji: "🟢",
            colors: vec![
                ("YellowGreen", [154, 205, 50]),
                ("LawnGreen", [124, 252, 0]),
                ("Lime", [0, 255, 0]),
                ("LimeGreen", [50, 205, 50]),
                ("SpringGreen", [0, 255, 127]),
                ("MediumSpringGreen", [0, 250, 154]),
                ("LightGreen", [144, 238, 144]),
                ("PaleGreen", [152, 251, 152]),
                ("DarkGreen", [0, 100, 0]),
                ("Green", [0, 128, 0]),
                ("ForestGreen", [34, 139, 34]),
                ("SeaGreen", [46, 139, 87]),
                ("Olive", [128, 128, 0]),
                ("OliveDrab", [107, 142, 35]),
            ],
        },
        ColorGroup {
            name: "Cyans",
            emoji: "🩵",
            colors: vec![
                ("Cyan", [0, 255, 255]),
                ("Aqua", [0, 255, 255]),
                ("LightCyan", [224, 255, 255]),
                ("Turquoise", [64, 224, 208]),
                ("MediumTurquoise", [72, 209, 204]),
                ("DarkTurquoise", [0, 206, 209]),
                ("Teal", [0, 128, 128]),
            ],
        },
        ColorGroup {
            name: "Blues",
            emoji: "🔵",
            colors: vec![
                ("LightBlue", [173, 216, 230]),
                ("SkyBlue", [135, 206, 235]),
                ("LightSkyBlue", [135, 206, 250]),
                ("DeepSkyBlue", [0, 191, 255]),
                ("DodgerBlue", [30, 144, 255]),
                ("CornflowerBlue", [100, 149, 237]),
                ("SteelBlue", [70, 130, 180]),
                ("RoyalBlue", [65, 105, 225]),
                ("Blue", [0, 0, 255]),
                ("MediumBlue", [0, 0, 205]),
                ("DarkBlue", [0, 0, 139]),
                ("Navy", [0, 0, 128]),
                ("MidnightBlue", [25, 25, 112]),
            ],
        },
        ColorGroup {
            name: "Purples",
            emoji: "🟣",
            colors: vec![
                ("Lavender", [230, 230, 250]),
                ("Thistle", [216, 191, 216]),
                ("Plum", [221, 160, 221]),
                ("Violet", [238, 130, 238]),
                ("Orchid", [218, 112, 214]),
                ("Magenta", [255, 0, 255]),
                ("MediumOrchid", [186, 85, 211]),
                ("MediumPurple", [147, 112, 219]),
                ("BlueViolet", [138, 43, 226]),
                ("DarkViolet", [148, 0, 211]),
                ("DarkOrchid", [153, 50, 204]),
                ("DarkMagenta", [139, 0, 139]),
                ("Purple", [128, 0, 128]),
                ("Indigo", [75, 0, 130]),
                ("SlateBlue", [106, 90, 205]),
                ("DarkSlateBlue", [72, 61, 139]),
            ],
        },
        ColorGroup {
            name: "Pinks",
            emoji: "🩷",
            colors: vec![
                ("Pink", [255, 192, 203]),
                ("LightPink", [255, 182, 193]),
                ("HotPink", [255, 105, 180]),
                ("DeepPink", [255, 20, 147]),
                ("MediumVioletRed", [199, 21, 133]),
                ("PaleVioletRed", [219, 112, 147]),
            ],
        },
        ColorGroup {
            name: "Browns",
            emoji: "🟤",
            colors: vec![
                ("Cornsilk", [255, 248, 220]),
                ("BlanchedAlmond", [255, 235, 205]),
                ("Bisque", [255, 228, 196]),
                ("Wheat", [245, 222, 179]),
                ("BurlyWood", [222, 184, 135]),
                ("Tan", [210, 180, 140]),
                ("RosyBrown", [188, 143, 143]),
                ("SandyBrown", [244, 164, 96]),
                ("Goldenrod", [218, 165, 32]),
                ("Peru", [205, 133, 63]),
                ("Chocolate", [210, 105, 30]),
                ("SaddleBrown", [139, 69, 19]),
                ("Sienna", [160, 82, 45]),
                ("Brown", [165, 42, 42]),
                ("Maroon", [128, 0, 0]),
            ],
        },
        ColorGroup {
            name: "Whites",
            emoji: "⚪",
            colors: vec![
                ("White", [255, 255, 255]),
                ("Snow", [255, 250, 250]),
                ("Honeydew", [240, 255, 240]),
                ("MintCream", [245, 255, 250]),
                ("Azure", [240, 255, 255]),
                ("AliceBlue", [240, 248, 255]),
                ("GhostWhite", [248, 248, 255]),
                ("WhiteSmoke", [245, 245, 245]),
                ("Seashell", [255, 245, 238]),
                ("Beige", [245, 245, 220]),
                ("OldLace", [253, 245, 230]),
                ("FloralWhite", [255, 250, 240]),
                ("Ivory", [255, 255, 240]),
                ("AntiqueWhite", [250, 235, 215]),
                ("Linen", [250, 240, 230]),
                ("LavenderBlush", [255, 240, 245]),
                ("MistyRose", [255, 228, 225]),
            ],
        },
        ColorGroup {
            name: "Grays",
            emoji: "⚫",
            colors: vec![
                ("Gainsboro", [220, 220, 220]),
                ("LightGray", [211, 211, 211]),
                ("Silver", [192, 192, 192]),
                ("DarkGray", [169, 169, 169]),
                ("Gray", [128, 128, 128]),
                ("DimGray", [105, 105, 105]),
                ("LightSlateGray", [119, 136, 153]),
                ("SlateGray", [112, 128, 144]),
                ("DarkSlateGray", [47, 79, 79]),
                ("Black", [0, 0, 0]),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spaced_names() {
        assert_eq!(to_spaced_name("Red"), "Red");
        assert_eq!(to_spaced_name("DarkRed"), "Dark Red");
        assert_eq!(to_spaced_name("SlateBlue"), "Slate Blue");
        assert_eq!(to_spaced_name("MediumSpringGreen"), "Medium Spring Green");
    }
}
