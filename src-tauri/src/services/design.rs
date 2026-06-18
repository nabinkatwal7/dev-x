use crate::error::AppError;
use crate::models::{CommandExecutionResult, CommandExecutionStatus};
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn eyedropper(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "design.eyedropper",
            "Precision Eyedropper",
            "Paste a sampled color value like #2E6BFF, rgb(46,107,255), or hsl(222,100%,59%). The tool normalizes it to hex, rgb, hsl, and Swift UIColor components.".into(),
            "Provide a color sample to normalize.",
        ));
    }

    let color = parse_color(trimmed)?;
    Ok(success(
        "design.eyedropper",
        "Precision Eyedropper",
        color_report(&color),
        "Normalized sampled color.",
    ))
}

pub fn color_swapper(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "design.color-swap",
            "Color Swapper",
            "Paste a color in hex, rgb(), or hsl() form. Output includes hex, rgb, hsl, cmyk, and CSS-friendly forms.".into(),
            "Provide a color value to convert.",
        ));
    }

    let color = parse_color(trimmed)?;
    Ok(success(
        "design.color-swap",
        "Color Swapper",
        full_color_report(&color),
        "Converted color formats.",
    ))
}

pub fn svg_optimize(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "design.svg-optimize",
            "SVG Optimizer",
            "Paste SVG markup to strip comments, XML headers, metadata, repeated whitespace, and redundant newlines.".into(),
            "Provide SVG markup to optimize.",
        ));
    }

    let mut svg = trimmed.replace("<?xml version=\"1.0\" encoding=\"UTF-8\"?>", "");
    svg = remove_xml_comments(&svg);
    svg = svg.replace(">\n<", "><");
    svg = svg
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" <", "<")
        .replace("> ", ">");

    Ok(success(
        "design.svg-optimize",
        "SVG Optimizer",
        svg,
        "Optimized SVG markup.",
    ))
}

pub fn typography_scale(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "design.type-scale",
            "Typography Scale",
            "Format:\nbase=16\nratio=1.25\nsizes=12,14,16,20,24,32".into(),
            "Provide a base size and optional sizes/ratio.",
        ));
    }

    let mut base = 16.0f64;
    let mut ratio = 1.25f64;
    let mut sizes = vec![12.0, 14.0, 16.0, 20.0, 24.0, 32.0];
    for line in trimmed.lines() {
        if let Some(value) = line.strip_prefix("base=") {
            base = value.trim().parse::<f64>().unwrap_or(16.0);
        } else if let Some(value) = line.strip_prefix("ratio=") {
            ratio = value.trim().parse::<f64>().unwrap_or(1.25);
        } else if let Some(value) = line.strip_prefix("sizes=") {
            sizes = value
                .split(',')
                .filter_map(|item| item.trim().parse::<f64>().ok())
                .collect();
        }
    }

    let mut output = String::from("px | rem | em | modular-step\n");
    for size in sizes {
        let rem = size / base;
        let step = if size > 0.0 { (size / base).ln() / ratio.ln() } else { 0.0 };
        output.push_str(&format!("{:.0} | {:.3}rem | {:.3}em | {:.2}\n", size, rem, rem, step));
    }

    Ok(success(
        "design.type-scale",
        "Typography Scale",
        output,
        "Computed proportional type scale.",
    ))
}

pub fn aspect_ratio(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "design.aspect-ratio",
            "Aspect Ratio Sandbox",
            "Paste width x height like 1920x1080 or 1080/1350.".into(),
            "Provide dimensions to analyze.",
        ));
    }

    let (width, height) = parse_dimensions(trimmed)?;
    let gcd = greatest_common_divisor(width as u64, height as u64).max(1) as f64;
    let ratio = format!("{}:{}", (width / gcd).round(), (height / gcd).round());
    let output = format!(
        "Width: {:.0}\nHeight: {:.0}\nRatio: {}\nDecimal: {:.4}\nScaled heights:\n320w -> {:.2}\n768w -> {:.2}\n1440w -> {:.2}",
        width,
        height,
        ratio,
        width / height,
        320.0 * height / width,
        768.0 * height / width,
        1440.0 * height / width
    );

    Ok(success(
        "design.aspect-ratio",
        "Aspect Ratio Sandbox",
        output,
        "Computed aspect ratio and scaling.",
    ))
}

pub fn layout_constructor(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "design.layout-css",
            "Flex/Grid Constructor",
            "Format:\nmode=flex|grid\ngap=16\ncolumns=3\nalign=center\njustify=space-between".into(),
            "Provide layout parameters.",
        ));
    }

    let mut mode = String::from("flex");
    let mut gap = String::from("1rem");
    let mut columns = 3usize;
    let mut align = String::from("stretch");
    let mut justify = String::from("flex-start");
    for line in trimmed.lines() {
        if let Some(value) = line.strip_prefix("mode=") {
            mode = value.trim().to_lowercase();
        } else if let Some(value) = line.strip_prefix("gap=") {
            gap = normalize_css_unit(value.trim());
        } else if let Some(value) = line.strip_prefix("columns=") {
            columns = value.trim().parse::<usize>().unwrap_or(3).max(1);
        } else if let Some(value) = line.strip_prefix("align=") {
            align = value.trim().into();
        } else if let Some(value) = line.strip_prefix("justify=") {
            justify = value.trim().into();
        }
    }

    let css = if mode == "grid" {
        format!(
            ".layout {{\n  display: grid;\n  grid-template-columns: repeat({}, minmax(0, 1fr));\n  gap: {};\n  align-items: {};\n  justify-items: {};\n}}",
            columns, gap, align, justify
        )
    } else {
        format!(
            ".layout {{\n  display: flex;\n  gap: {};\n  align-items: {};\n  justify-content: {};\n  flex-wrap: wrap;\n}}",
            gap, align, justify
        )
    };

    Ok(success(
        "design.layout-css",
        "Flex/Grid Constructor",
        css,
        format!("Generated {} layout CSS.", mode),
    ))
}

pub fn mock_data(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    let mut count = 5usize;
    let mut dataset = String::from("people");
    for line in trimmed.lines() {
        if let Some(value) = line.strip_prefix("count=") {
            count = value.trim().parse::<usize>().unwrap_or(5).clamp(1, 100);
        } else if let Some(value) = line.strip_prefix("dataset=") {
            dataset = value.trim().to_lowercase();
        } else if trimmed.parse::<usize>().is_ok() {
            count = trimmed.parse::<usize>().unwrap_or(5).clamp(1, 100);
        }
    }

    let items = match dataset.as_str() {
        "issues" => generate_issue_mock_data(count),
        "projects" => generate_project_mock_data(count),
        _ => generate_people_mock_data(count),
    };

    Ok(success(
        "design.mock-data",
        "Mock Data Engine",
        format!("[\n  {}\n]", items.join(",\n  ")),
        format!("Generated {} {} record(s).", count, dataset),
    ))
}

pub fn contrast_check(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "design.contrast",
            "Contrast Ratio",
            "Format:\nfg=#111827\nbg=#F9FAFB".into(),
            "Provide foreground and background colors.",
        ));
    }

    let mut fg = None;
    let mut bg = None;
    for line in trimmed.lines() {
        if let Some(value) = line.strip_prefix("fg=") {
            fg = Some(parse_color(value.trim())?);
        } else if let Some(value) = line.strip_prefix("bg=") {
            bg = Some(parse_color(value.trim())?);
        }
    }
    let fg = fg.ok_or_else(|| AppError::Internal("missing fg=<color>".into()))?;
    let bg = bg.ok_or_else(|| AppError::Internal("missing bg=<color>".into()))?;

    let ratio = contrast_ratio(&fg, &bg);
    let output = format!(
        "Foreground: {}\nBackground: {}\nContrast ratio: {:.2}:1\nWCAG AA normal text: {}\nWCAG AAA normal text: {}\nWCAG AA large text: {}",
        rgb_string(&fg),
        rgb_string(&bg),
        ratio,
        yes_no(ratio >= 4.5),
        yes_no(ratio >= 7.0),
        yes_no(ratio >= 3.0)
    );

    Ok(success(
        "design.contrast",
        "Contrast Ratio",
        output,
        "Computed WCAG contrast compliance.",
    ))
}

pub fn shadow_gradient(input: &str) -> Result<CommandExecutionResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(info(
            "design.shadow-gradient",
            "Shadow & Gradient Generator",
            "Format examples:\nshadow=soft color=#0f172a opacity=0.18\nor\ngradient=#0ea5e9,#1d4ed8 angle=135".into(),
            "Provide shadow or gradient parameters.",
        ));
    }

    if trimmed.contains("gradient=") {
        let mut colors = vec!["#0ea5e9".to_string(), "#1d4ed8".to_string()];
        let mut angle = 135i32;
        for line in trimmed.lines() {
            if let Some(value) = line.strip_prefix("gradient=") {
                colors = value.split(',').map(|item| item.trim().into()).collect();
            } else if let Some(value) = line.strip_prefix("angle=") {
                angle = value.trim().parse::<i32>().unwrap_or(135);
            }
        }
        let output = format!(
            "background: linear-gradient({}deg, {});",
            angle,
            colors.join(", ")
        );
        return Ok(success(
            "design.shadow-gradient",
            "Gradient Generator",
            output,
            "Generated gradient CSS.",
        ));
    }

    let mut shadow_kind = String::from("soft");
    let mut color = parse_color("#0f172a")?;
    let mut opacity = 0.18f64;
    for line in trimmed.lines() {
        if let Some(value) = line.strip_prefix("shadow=") {
            shadow_kind = value.trim().into();
        } else if let Some(value) = line.strip_prefix("color=") {
            color = parse_color(value.trim())?;
        } else if let Some(value) = line.strip_prefix("opacity=") {
            opacity = value.trim().parse::<f64>().unwrap_or(0.18);
        }
    }
    let rgba = format!("rgba({}, {}, {}, {:.2})", color.r, color.g, color.b, opacity);
    let css = match shadow_kind.as_str() {
        "crisp" => format!("box-shadow: 0 1px 2px {}, 0 6px 12px {};", rgba, rgba),
        "deep" => format!("box-shadow: 0 12px 24px {}, 0 24px 56px {};", rgba, rgba),
        _ => format!("box-shadow: 0 8px 20px {}, 0 2px 6px {};", rgba, rgba),
    };

    Ok(success(
        "design.shadow-gradient",
        "Shadow Generator",
        css,
        "Generated shadow CSS.",
    ))
}

pub fn font_inventory(_input: &str) -> Result<CommandExecutionResult, AppError> {
    let fonts_dir = env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\Windows"))
        .join("Fonts");
    let mut fonts = fs::read_dir(&fonts_dir)
        .map_err(|error| AppError::Internal(format!("failed to read {}: {}", fonts_dir.display(), error)))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().to_str().map(|name| name.to_string()))
        .collect::<Vec<_>>();
    fonts.sort();

    let mut by_extension = std::collections::BTreeMap::new();
    let mut families = std::collections::BTreeMap::new();
    for font in &fonts {
        if let Some((family, extension)) = font.rsplit_once('.') {
            *by_extension.entry(extension.to_lowercase()).or_insert(0usize) += 1;
            let family_key = family
                .trim_end_matches(|char: char| char.is_ascii_digit())
                .trim_end_matches('-')
                .to_string();
            *families.entry(family_key).or_insert(0usize) += 1;
        }
    }

    let family_sample = families
        .iter()
        .take(25)
        .map(|(family, count)| format!("{} ({})", family, count))
        .collect::<Vec<_>>()
        .join("\n");
    let extension_summary = by_extension
        .iter()
        .map(|(extension, count)| format!("{}.{} -> {}", extension, extension, count))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(success(
        "design.font-inventory",
        "Font Inventory",
        format!(
            "Fonts directory: {}\nFont files: {}\n\nFamilies:\n{}\n\nExtensions:\n{}",
            fonts_dir.display(),
            fonts.len(),
            family_sample,
            extension_summary
        ),
        format!("Discovered {} font file(s).", fonts.len()),
    ))
}

fn generate_people_mock_data(count: usize) -> Vec<String> {
    let first_names = ["Ava", "Noah", "Mina", "Theo", "Sara", "Leo", "Nina", "Omar"];
    let last_names = ["Patel", "Rivera", "Chen", "Miller", "Singh", "Nguyen", "Garcia", "Kim"];
    let roles = ["Designer", "Frontend Engineer", "Product Manager", "QA Analyst", "DevOps Engineer"];
    (0..count)
        .map(|index| {
            let first = first_names[index % first_names.len()];
            let last = last_names[(index * 3) % last_names.len()];
            let full = format!("{} {}", first, last);
            format!(
                "{{\"id\":{},\"name\":\"{}\",\"email\":\"{}.{}@example.dev\",\"role\":\"{}\",\"timezone\":\"UTC+{}\",\"avatar\":\"https://api.dicebear.com/7.x/identicon/svg?seed={}\"}}",
                index + 1,
                full,
                first.to_lowercase(),
                last.to_lowercase(),
                roles[index % roles.len()],
                (index % 9) + 1,
                full.replace(' ', "-").to_lowercase()
            )
        })
        .collect()
}

fn generate_issue_mock_data(count: usize) -> Vec<String> {
    let labels = ["bug", "infra", "ui", "docs", "security"];
    let statuses = ["open", "in_progress", "blocked", "closed"];
    (0..count)
        .map(|index| {
            format!(
                "{{\"id\":\"ISSUE-{:03}\",\"title\":\"Fix workflow edge case {}\",\"status\":\"{}\",\"priority\":\"P{}\",\"label\":\"{}\",\"estimateHours\":{}}}",
                index + 1,
                index + 1,
                statuses[index % statuses.len()],
                (index % 4) + 1,
                labels[index % labels.len()],
                (index % 6) + 1
            )
        })
        .collect()
}

fn generate_project_mock_data(count: usize) -> Vec<String> {
    let stages = ["planning", "active", "review", "shipped"];
    (0..count)
        .map(|index| {
            format!(
                "{{\"id\":\"PROJ-{:03}\",\"name\":\"Platform Track {}\",\"stage\":\"{}\",\"owner\":\"team-{}\",\"budgetUsd\":{},\"health\":\"{}\"}}",
                index + 1,
                index + 1,
                stages[index % stages.len()],
                (index % 5) + 1,
                15000 + (index as i32 * 2400),
                if index % 5 == 0 { "watch" } else { "good" }
            )
        })
        .collect()
}

#[derive(Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

fn parse_color(input: &str) -> Result<Color, AppError> {
    let input = input.trim();
    if let Some(hex) = input.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    if input.to_lowercase().starts_with("rgb(") {
        let values = input[4..input.len() - 1]
            .split(',')
            .map(|part| part.trim().parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| AppError::Internal(format!("invalid rgb color: {}", error)))?;
        if values.len() != 3 {
            return Err(AppError::Internal("rgb() requires three components".into()));
        }
        return Ok(Color {
            r: values[0],
            g: values[1],
            b: values[2],
        });
    }
    if input.to_lowercase().starts_with("hsl(") {
        let raw = input[4..input.len() - 1].split(',').map(|part| part.trim()).collect::<Vec<_>>();
        if raw.len() != 3 {
            return Err(AppError::Internal("hsl() requires three components".into()));
        }
        let h = raw[0].parse::<f64>().map_err(|error| AppError::Internal(format!("invalid hue: {}", error)))?;
        let s = raw[1].trim_end_matches('%').parse::<f64>().map_err(|error| AppError::Internal(format!("invalid saturation: {}", error)))? / 100.0;
        let l = raw[2].trim_end_matches('%').parse::<f64>().map_err(|error| AppError::Internal(format!("invalid lightness: {}", error)))? / 100.0;
        return Ok(hsl_to_rgb(h, s, l));
    }
    Err(AppError::Internal("unsupported color format".into()))
}

fn parse_hex_color(hex: &str) -> Result<Color, AppError> {
    match hex.len() {
        3 => {
            let chars = hex.chars().collect::<Vec<_>>();
            Ok(Color {
                r: u8::from_str_radix(&format!("{}{}", chars[0], chars[0]), 16).map_err(hex_error)?,
                g: u8::from_str_radix(&format!("{}{}", chars[1], chars[1]), 16).map_err(hex_error)?,
                b: u8::from_str_radix(&format!("{}{}", chars[2], chars[2]), 16).map_err(hex_error)?,
            })
        }
        6 => Ok(Color {
            r: u8::from_str_radix(&hex[0..2], 16).map_err(hex_error)?,
            g: u8::from_str_radix(&hex[2..4], 16).map_err(hex_error)?,
            b: u8::from_str_radix(&hex[4..6], 16).map_err(hex_error)?,
        }),
        _ => Err(AppError::Internal("hex colors must be #RGB or #RRGGBB".into())),
    }
}

fn hex_error(error: std::num::ParseIntError) -> AppError {
    AppError::Internal(format!("invalid hex color: {}", error))
}

fn color_report(color: &Color) -> String {
    let (h, s, l) = rgb_to_hsl(*color);
    format!(
        "Hex: #{:02X}{:02X}{:02X}\nRGB: rgb({}, {}, {})\nHSL: hsl({:.0}, {:.0}%, {:.0}%)\nSwift: UIColor(red: {:.3}, green: {:.3}, blue: {:.3}, alpha: 1.0)",
        color.r,
        color.g,
        color.b,
        color.r,
        color.g,
        color.b,
        h,
        s * 100.0,
        l * 100.0,
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0
    )
}

fn full_color_report(color: &Color) -> String {
    let (c, m, y, k) = rgb_to_cmyk(*color);
    format!(
        "{}\nCMYK: {:.0}%, {:.0}%, {:.0}%, {:.0}%\nCSS vars:\n--color: #{:02X}{:02X}{:02X};\ncolor: rgb({}, {}, {});",
        color_report(color),
        c * 100.0,
        m * 100.0,
        y * 100.0,
        k * 100.0,
        color.r,
        color.g,
        color.b,
        color.r,
        color.g,
        color.b
    )
}

fn rgb_to_hsl(color: Color) -> (f64, f64, f64) {
    let r = color.r as f64 / 255.0;
    let g = color.g as f64 / 255.0;
    let b = color.b as f64 / 255.0;
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;
    let l = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, l);
    }

    let s = delta / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    (if h < 0.0 { h + 360.0 } else { h }, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Color {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color {
        r: ((r1 + m) * 255.0).round() as u8,
        g: ((g1 + m) * 255.0).round() as u8,
        b: ((b1 + m) * 255.0).round() as u8,
    }
}

fn rgb_to_cmyk(color: Color) -> (f64, f64, f64, f64) {
    let r = color.r as f64 / 255.0;
    let g = color.g as f64 / 255.0;
    let b = color.b as f64 / 255.0;
    let k = 1.0 - r.max(g.max(b));
    if k >= 1.0 {
        return (0.0, 0.0, 0.0, 1.0);
    }
    (
        (1.0 - r - k) / (1.0 - k),
        (1.0 - g - k) / (1.0 - k),
        (1.0 - b - k) / (1.0 - k),
        k,
    )
}

fn parse_dimensions(input: &str) -> Result<(f64, f64), AppError> {
    if let Some((left, right)) = input.split_once('x') {
        return Ok((
            left.trim().parse::<f64>().map_err(|error| AppError::Internal(format!("invalid width: {}", error)))?,
            right.trim().parse::<f64>().map_err(|error| AppError::Internal(format!("invalid height: {}", error)))?,
        ));
    }
    if let Some((left, right)) = input.split_once('/') {
        return Ok((
            left.trim().parse::<f64>().map_err(|error| AppError::Internal(format!("invalid width: {}", error)))?,
            right.trim().parse::<f64>().map_err(|error| AppError::Internal(format!("invalid height: {}", error)))?,
        ));
    }
    Err(AppError::Internal("dimensions must use widthxheight or width/height".into()))
}

fn greatest_common_divisor(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = a % b;
        a = b;
        b = temp;
    }
    a
}

fn normalize_css_unit(value: &str) -> String {
    if value.ends_with("px") || value.ends_with("rem") || value.ends_with("em") || value.ends_with('%') {
        value.into()
    } else {
        format!("{}px", value)
    }
}

fn contrast_ratio(left: &Color, right: &Color) -> f64 {
    let l1 = relative_luminance(left);
    let l2 = relative_luminance(right);
    let (bright, dark) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (bright + 0.05) / (dark + 0.05)
}

fn relative_luminance(color: &Color) -> f64 {
    let channel = |value: u8| {
        let v = value as f64 / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

fn rgb_string(color: &Color) -> String {
    format!("rgb({}, {}, {})", color.r, color.g, color.b)
}

fn yes_no(value: bool) -> &'static str {
    if value { "pass" } else { "fail" }
}

fn remove_xml_comments(input: &str) -> String {
    let mut output = String::new();
    let mut remaining = input;
    while let Some(start) = remaining.find("<!--") {
        output.push_str(&remaining[..start]);
        if let Some(end) = remaining[start + 4..].find("-->") {
            remaining = &remaining[start + 4 + end + 3..];
        } else {
            break;
        }
    }
    output.push_str(remaining);
    output
}

fn info(command_id: &str, title: &str, output: String, summary: impl Into<String>) -> CommandExecutionResult {
    CommandExecutionResult {
        command_id: command_id.into(),
        title: title.into(),
        output,
        status: CommandExecutionStatus::Info,
        summary: summary.into(),
    }
}

fn success(command_id: &str, title: &str, output: String, summary: impl Into<String>) -> CommandExecutionResult {
    CommandExecutionResult {
        command_id: command_id.into(),
        title: title.into(),
        output,
        status: CommandExecutionStatus::Success,
        summary: summary.into(),
    }
}
