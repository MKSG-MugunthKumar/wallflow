#!/usr/bin/env swift
//
// wallflow_helper.swift
// Native macOS wallpaper setter for wallflow
//
// Usage: wallflow_helper <image_path> [scaling] [screen]
//   scaling: fill (default), fit, stretch, center
//   screen: all (default), main, <index>
//
// This helper sets wallpaper on ALL Spaces (virtual desktops), not just
// the current one, by using AppleScript via NSAppleScript.
//

import Cocoa

// MARK: - Wallpaper Setting

/// Set wallpaper using AppleScript (works across all Spaces)
func setWallpaperViaAppleScript(imagePath: String) -> Bool {
    // This AppleScript sets the wallpaper on ALL desktops (all Spaces)
    let script = """
    tell application "System Events"
        tell every desktop
            set picture to "\(imagePath)"
        end tell
    end tell
    """

    var error: NSDictionary?
    if let appleScript = NSAppleScript(source: script) {
        appleScript.executeAndReturnError(&error)
        if let error = error {
            fputs("AppleScript error: \(error)\n", stderr)
            return false
        }
        return true
    }
    return false
}

/// Set wallpaper using NSWorkspace (only affects current Space per screen)
func setWallpaperViaNSWorkspace(imagePath: String, scaling: String, screenSelection: String) -> Bool {
    let url = URL(fileURLWithPath: imagePath)

    // Determine scaling option
    var options: [NSWorkspace.DesktopImageOptionKey: Any] = [:]
    switch scaling {
    case "fill":
        options[.imageScaling] = NSImageScaling.scaleProportionallyUpOrDown.rawValue
        options[.allowClipping] = true
    case "fit":
        options[.imageScaling] = NSImageScaling.scaleProportionallyUpOrDown.rawValue
        options[.allowClipping] = false
    case "stretch":
        options[.imageScaling] = NSImageScaling.scaleAxesIndependently.rawValue
    case "center":
        options[.imageScaling] = NSImageScaling.scaleNone.rawValue
    default:
        options[.imageScaling] = NSImageScaling.scaleProportionallyUpOrDown.rawValue
        options[.allowClipping] = true
    }

    // Determine which screens to set
    let screens: [NSScreen]
    switch screenSelection {
    case "all":
        screens = NSScreen.screens
    case "main":
        screens = NSScreen.main.map { [$0] } ?? []
    default:
        if let index = Int(screenSelection), index < NSScreen.screens.count {
            screens = [NSScreen.screens[index]]
        } else {
            screens = NSScreen.screens
        }
    }

    // Set wallpaper for each screen
    var success = true
    for screen in screens {
        do {
            try NSWorkspace.shared.setDesktopImageURL(url, for: screen, options: options)
        } catch {
            fputs("Error setting wallpaper for screen: \(error)\n", stderr)
            success = false
        }
    }

    return success
}

// MARK: - Main

func printUsage() {
    fputs("""
    Usage: wallflow_helper <image_path> [scaling] [screen]

    Arguments:
        image_path  Path to the wallpaper image file
        scaling     Scaling mode: fill (default), fit, stretch, center
        screen      Screen selection: all (default), main, <index>

    Note: This helper sets wallpaper on ALL Spaces (virtual desktops).

    """, stderr)
}

func main() {
    let args = CommandLine.arguments

    guard args.count >= 2 else {
        printUsage()
        exit(1)
    }

    let imagePath = args[1]
    let scaling = args.count > 2 ? args[2] : "fill"
    let screen = args.count > 3 ? args[3] : "all"

    // Verify image exists
    guard FileManager.default.fileExists(atPath: imagePath) else {
        fputs("Error: Image file not found: \(imagePath)\n", stderr)
        exit(1)
    }

    // Method 1: Use AppleScript to set on ALL Spaces
    // This is the reliable way to set wallpaper across all virtual desktops
    let appleScriptSuccess = setWallpaperViaAppleScript(imagePath: imagePath)

    // Method 2: Also use NSWorkspace for the current Space (better scaling support)
    // This ensures proper scaling options are applied to the current space
    let nsWorkspaceSuccess = setWallpaperViaNSWorkspace(
        imagePath: imagePath,
        scaling: scaling,
        screenSelection: screen
    )

    if appleScriptSuccess || nsWorkspaceSuccess {
        print("Wallpaper set successfully")
        exit(0)
    } else {
        fputs("Failed to set wallpaper\n", stderr)
        exit(1)
    }
}

main()
