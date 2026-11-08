//! Android and iOS deep-link intake.
//!
//! On mobile the *registration* of URL schemes is declarative — Android
//! apps declare `<intent-filter>`s in `AndroidManifest.xml`, Apple apps
//! declare `CFBundleURLTypes` in `Info.plist` (or associated domains for
//! universal links). nexum cannot write those files at runtime; the
//! generators below emit the exact declarations for the configured
//! schemes:
//!
//! * `android_manifest_intent_filter` — the `<intent-filter>` block to
//!   paste into the `<activity>` of `AndroidManifest.xml`.
//! * [`url_types_xml`](crate::plist::url_types_xml) — the
//!   `CFBundleURLTypes` block for `Info.plist` (iOS and macOS).
//!
//! At *runtime* the UI framework (or the platform glue of the app) receives
//! the incoming URL and hands it to nexum through `push_intent_url`
//! (Android: `Intent.getDataString()` from `onNewIntent` / `onCreate`) or
//! `push_open_urls` (iOS: `openURLContexts` / `openURL` application
//! delegate callbacks). The URLs then flow through the regular hub, so
//! adapters and consumers work identically on mobile.

use std::fmt::Write as _;

use nexum_core::Config;

/// Renders the Android `<intent-filter>` that registers every configured
/// scheme for VIEW intents (browser + app links).
#[must_use]
pub fn android_manifest_intent_filter(config: &Config) -> String {
    let mut filter = String::from(
        "<intent-filter>\n    \
         <action android:name=\"android.intent.action.VIEW\" />\n    \
         <category android:name=\"android.intent.category.DEFAULT\" />\n    \
         <category android:name=\"android.intent.category.BROWSABLE\" />\n",
    );
    for scheme in &config.schemes {
        let _ = writeln!(filter, "    <data android:scheme=\"{scheme}\" />");
    }
    filter.push_str("</intent-filter>\n");
    filter
}

/// Feeds a URL received from an Android `Intent` into the global hub.
///
/// Call this from the JNI glue of your framework when
/// `Intent.getDataString()` yields a deep link. The URL is delivered as a
/// regular [`Source::System`](nexum_core::Source) event.
///
/// # Errors
///
/// Returns [`Error::HubNotSet`](nexum_core::Error) when the hub was not
/// initialized yet.
#[cfg(target_os = "android")]
pub fn push_intent_url(url: impl Into<String>) -> nexum_core::Result<()> {
    crate::push_url(url)
}

/// Feeds URLs received from the iOS application delegate into the global
/// hub.
///
/// Call this from your `openURLContexts(_:)` / `open URL:` delegate
/// implementation with every URL the system handed over. Each URL is
/// delivered as a regular [`Source::System`](nexum_core::Source) event.
///
/// # Errors
///
/// Returns [`Error::HubNotSet`](nexum_core::Error) when the hub was not
/// initialized yet.
#[cfg(target_os = "ios")]
pub fn push_open_urls<I>(urls: I) -> nexum_core::Result<()>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    for url in urls {
        crate::push_url(url)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intent_filter_declares_every_scheme() {
        let config = Config::new(["myapp", "beta"]);
        let filter = android_manifest_intent_filter(&config);
        assert!(filter.contains("<action android:name=\"android.intent.action.VIEW\" />"));
        assert!(filter.contains("<category android:name=\"android.intent.category.BROWSABLE\" />"));
        assert!(filter.contains("<data android:scheme=\"myapp\" />"));
        assert!(filter.contains("<data android:scheme=\"beta\" />"));
        assert!(filter.trim_end().ends_with("</intent-filter>"));
    }

    #[test]
    fn intent_filter_with_no_schemes_has_no_data_elements() {
        let filter = android_manifest_intent_filter(&Config::new([] as [&str; 0]));
        assert!(!filter.contains("<data"));
    }

    #[test]
    fn ios_url_types_match_the_macos_generator() {
        // One source of truth for Apple bundle declarations (DRY).
        let config = Config::new(["myapp"]);
        assert!(crate::plist::url_types_xml(&config).contains("<string>myapp</string>"));
    }
}
