// Test file with multiple targets in the same file
// This should test our new file-based update logic

// Multiple different packages on separate lines
const vscode = "1.0.0" // [cup] microsoft/vscode
const react = "17.0.0" // [cup] facebook/react
version_rust = 1.91.0 // [cup] rust-lang/rust

// Multiple versions of the same package on different lines
const node_version1 = "18.0.0" // [cup] nodejs/node
const node_version2 = "18.0.0" // [cup] nodejs/node

// Test different patterns on the same line (this tests replace_all)
// Note: Each [cup] annotation should be on its own line for proper parsing