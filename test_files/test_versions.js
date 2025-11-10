// Test file with different cup annotation formats

// Old format with explicit GitHub type
const version1 = "1.105.1" // [cup] GitHub microsoft/vscode

// New format without type (should fallback to remote_default)
const version2 = "19.2.0" // [cup] facebook/react

// Another example with assignment operator
version3 = 1.91.0 // [cup] rust-lang/rust

// JSON-like format without explicit type
"package": "25.1.0" // [cup] nodejs/node