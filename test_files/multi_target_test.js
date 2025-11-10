// Test file with multiple targets in the same file
// This should test our new file-based update logic

// Multiple different packages on separate lines
const vscode = "1.105.1" // [cup] microsoft/vscode
const react = "19.2.0" // [cup] facebook/react
version_rust = 1.91.0 // [cup] rust-lang/rust

// Multiple versions of the same package on different lines
const node_version1 = "25.1.0" // [cup] nodejs/node
const node_version2 = "25.1.0" // [cup] nodejs/node

// Test different patterns on the same line (this tests replace_all)
// Note: Each [cup] annotation should be on its own line for proper parsing