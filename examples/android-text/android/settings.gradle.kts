// The Android app around examples/android-text: a Kotlin host (crates/rune/android) and the
// Rust library cargo builds. scripts/android-apk.sh android-text builds both.
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}
rootProject.name = "android-text"
