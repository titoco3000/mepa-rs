use std::io;

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