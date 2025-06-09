use std::fs::File;
use std::io::{self, Write};

pub fn print_matrix(matrix: &Vec<Vec<String>>) {
    // Calculate the maximum width for each column
    let mut max_widths: Vec<usize> = Vec::with_capacity(matrix[0].len());

    for row in matrix {
        for (i, item) in row.iter().enumerate() {
            let item_len = format!("{}", item).len();
            if max_widths.get(i).is_some() {
                max_widths[i] = max_widths[i].max(item_len);
            } else {
                max_widths.push(item_len);
            }
        }
    }

    // Print the matrix with proper alignment
    for row in matrix {
        for (i, item) in row.iter().enumerate() {
            let width = max_widths.get(i).unwrap_or(&0);
            print!("{:width$} ", item, width = width);
        }
        println!();
    }
}

pub fn matrix_to_string(matrix: &Vec<Vec<String>>) -> String {
    if matrix.is_empty() {
        return String::new();
    }

    // Calculate the maximum width for each column
    let mut max_widths: Vec<usize> = Vec::new();
    for row in matrix {
        for (i, item) in row.iter().enumerate() {
            let item_len = item.len();
            if i < max_widths.len() {
                max_widths[i] = max_widths[i].max(item_len);
            } else {
                max_widths.push(item_len);
            }
        }
    }

    let mut output = String::new();
    // Build the string with proper alignment
    for row in matrix {
        for (i, item) in row.iter().enumerate() {
            if let Some(width) = max_widths.get(i) {
                // Left-align the item within its column width, followed by a space
                output.push_str(&format!("{:<width$} ", item, width = *width));
            }
        }
        // Replace the trailing space with a newline for a clean line ending
        if !row.is_empty() {
            output.pop();
        }
        output.push('\n');
    }
    output
}

pub fn write_matrix(matrix: &Vec<Vec<String>>, file: File) -> io::Result<()> {
    // Calculate the maximum width for each column
    let mut max_widths: Vec<usize> = Vec::with_capacity(matrix[0].len());

    for row in matrix {
        for (i, item) in row.iter().enumerate() {
            let item_len = format!("{}", item).len();
            if max_widths.get(i).is_some() {
                max_widths[i] = max_widths[i].max(item_len);
            } else {
                max_widths.push(item_len);
            }
        }
    }

    // Write the matrix with proper alignment
    for row in matrix {
        for (i, item) in row.iter().enumerate() {
            let width = max_widths.get(i).unwrap_or(&0);
            write!(&file, "{:width$} ", item, width = width)?;
        }
        writeln!(&file)?;
    }
    Ok(())
}

pub fn input_i32() -> i32 {
    let mut input_line = String::new();
    let mut retry = false;
    loop {
        println!("Type an {}integer: ", if retry { "valid " } else { "" });
        io::stdin()
            .read_line(&mut input_line)
            .expect("Failed to read line");
        if let Ok(v) = input_line.trim().parse() {
            return v;
        }
        retry = true;
    }
}
