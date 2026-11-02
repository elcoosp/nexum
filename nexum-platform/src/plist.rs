//! `Info.plist` helpers: generating and validating the `CFBundleURLTypes`
//! declaration that makes macOS (and iOS) deliver deep links to a bundle.
//!
//! The pure functions in this module are compiled and tested on every
//! platform; they power the macOS bundle validator, the iOS/Android setup
//! generators ([`crate::mobile`]) and the documentation.

use std::fmt::Write as _;

use nexum_core::Config;

/// Renders the `CFBundleURLTypes` array for the configured schemes as XML,
/// in the exact shape an `Info.plist` (or an iOS `Info.plist`) expects.
///
/// Use it to scaffold the bundle configuration for a new application or to
/// diff against an existing bundle with [`missing_schemes`].
#[must_use]
pub fn url_types_xml(config: &Config) -> String {
    let mut types = String::from(
        "<key>CFBundleURLTypes</key>\n<array>\n<dict>\n<key>CFBundleURLName</key>\n\
         <string>com.nexum.url</string>\n<key>CFBundleURLSchemes</key>\n<array>\n",
    );
    for scheme in &config.schemes {
        let _ = writeln!(types, "<string>{scheme}</string>");
    }
    types.push_str("</array>\n</dict>\n</array>\n");
    types
}

/// Returns the configured schemes that the given `Info.plist` content does
/// **not** declare.
///
/// The comparison is case-insensitive because URL schemes are
/// case-insensitive per RFC 3986 and Apple lower-cases the values it
/// writes. The function parses only the `CFBundleURLSchemes` strings, so
/// it tolerates arbitrary plist formatting (whitespace, comments, other
/// keys).
#[must_use]
pub fn missing_schemes(plist_xml: &str, config: &Config) -> Vec<String> {
    diff_schemes(&extract_schemes(plist_xml), config)
}

/// Returns the configured schemes missing from `present` (case-insensitive).
fn diff_schemes(present: &[String], config: &Config) -> Vec<String> {
    config
        .schemes
        .iter()
        .filter(|scheme| {
            let scheme = scheme.to_ascii_lowercase();
            !present
                .iter()
                .any(|declared| declared.to_ascii_lowercase() == scheme)
        })
        .cloned()
        .collect()
}

/// Extracts every `CFBundleURLSchemes` entry from raw plist XML.
fn extract_schemes(plist_xml: &str) -> Vec<String> {
    let mut schemes = Vec::new();
    let Some(position) = plist_xml.find("CFBundleURLSchemes") else {
        return schemes;
    };
    // The <array> following the CFBundleURLSchemes key holds the schemes.
    let Some(array_start) = plist_xml[position..].find("<array>") else {
        return schemes;
    };
    let array = &plist_xml[position + array_start..];
    let Some(array_end) = array.find("</array>") else {
        return schemes;
    };
    let mut cursor = &array[..array_end];
    while let Some(tag_start) = cursor.find("<string>") {
        let rest = &cursor[tag_start + "<string>".len()..];
        let Some(tag_end) = rest.find("</string>") else {
            break;
        };
        schemes.push(rest[..tag_end].to_owned());
        cursor = &rest[tag_end + "</string>".len()..];
    }
    schemes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(schemes: &[&str]) -> Config {
        Config::new(schemes.iter().copied())
    }

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.example.myapp</string>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>com.example.myapp</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>myapp</string>
                <string>MyApp-Beta</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#;

    #[test]
    fn url_types_xml_lists_every_scheme() {
        let xml = url_types_xml(&config(&["myapp", "beta"]));
        assert!(xml.contains("<key>CFBundleURLTypes</key>"));
        assert!(xml.contains("<string>myapp</string>"));
        assert!(xml.contains("<string>beta</string>"));
        // Ends with the closing array/dict structure.
        assert!(xml.trim_end().ends_with("</array>"));
    }

    #[test]
    fn url_types_xml_with_no_schemes_is_an_empty_array() {
        let xml = url_types_xml(&config(&[]));
        assert!(xml.contains("<array>\n</array>"));
    }

    #[test]
    fn missing_schemes_detects_missing_and_ignores_case() {
        // `MyApp-Beta` is declared with different casing: it counts.
        assert_eq!(
            missing_schemes(SAMPLE, &config(&["myapp", "myapp-beta"])),
            Vec::<String>::new()
        );
        // `missing` is not declared anywhere.
        assert_eq!(
            missing_schemes(SAMPLE, &config(&["myapp", "missing"])),
            vec!["missing".to_owned()]
        );
    }

    #[test]
    fn missing_schemes_handles_broken_or_empty_plists() {
        assert_eq!(
            missing_schemes("", &config(&["myapp"])),
            vec!["myapp".to_owned()],
            "no CFBundleURLSchemes at all"
        );
        assert_eq!(
            missing_schemes("<key>CFBundleURLSchemes</key>", &config(&["myapp"])),
            vec!["myapp".to_owned()],
            "key without array"
        );
        assert_eq!(
            missing_schemes(
                "<key>CFBundleURLSchemes</key>\n<array>\n<string>unterminated",
                &config(&["myapp"])
            ),
            vec!["myapp".to_owned()],
            "array without closing tag"
        );
        assert_eq!(
            missing_schemes(
                "<key>CFBundleURLSchemes</key>\n<array>\n<string>myapp\n</array>",
                &config(&["myapp"])
            ),
            vec!["myapp".to_owned()],
            "string tag without closing tag"
        );
        assert_eq!(missing_schemes(SAMPLE, &config(&[])), Vec::<String>::new());
    }
}
