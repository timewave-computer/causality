//! Utility functions for string formatting and manipulation

use std::fmt;

/// Format an object with debug formatting, returning a string
pub fn debug_format<T: fmt::Debug>(obj: &T) -> String {
    format!("{:?}", obj)
}

/// Format an object with display formatting, returning a string
pub fn display_format<T: fmt::Display>(obj: &T) -> String {
    format!("{}", obj)
}

/// Truncate a string to a given length, adding "..." if truncated
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut truncated = s.chars().take(max_len - 3).collect::<String>();
        truncated.push_str("...");
        truncated
    }
}

/// Split a string into lines and truncate to a maximum number of lines
pub fn truncate_lines(s: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= max_lines {
        s.to_string()
    } else {
        let mut truncated = lines.iter().take(max_lines - 1).fold(String::new(), |mut acc, line| {
            acc.push_str(line);
            acc.push('\n');
            acc
        });
        truncated.push_str("...");
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_format() {
        #[derive(Debug)]
        struct TestStruct {
            field: i32,
        }
        
        let obj = TestStruct { field: 42 };
        assert_eq!(debug_format(&obj), "TestStruct { field: 42 }");
    }
    
    #[test]
    fn test_display_format() {
        struct TestStruct {
            field: i32,
        }
        
        impl fmt::Display for TestStruct {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Value: {}", self.field)
            }
        }
        
        let obj = TestStruct { field: 42 };
        assert_eq!(display_format(&obj), "Value: 42");
    }
    
    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("Hello, world!", 20), "Hello, world!");
        assert_eq!(truncate_str("Hello, world!", 8), "Hello...");
    }
    
    #[test]
    fn test_truncate_lines() {
        let multiline = "Line 1\nLine 2\nLine 3\nLine 4";
        assert_eq!(truncate_lines(multiline, 5), multiline);
        assert_eq!(truncate_lines(multiline, 3), "Line 1\nLine 2\n...");
    }
} 