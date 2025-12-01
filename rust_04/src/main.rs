use clap::Parser;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::thread;
use std::time::Duration;

/// Hex Grid Pathfinding - Dijkstra
#[derive(Parser, Debug)]
#[command(name = "hexpath")]
struct Args {
    /// Map file (hex values, space separated)
    map_file: Option<String>,

    /// Generate random map (e.g., 8x4, 10x10)
    #[arg(long)]
    generate: Option<String>,

    /// Save generated map to file
    #[arg(long)]
    output: Option<String>,

    /// Show colored map
    #[arg(long)]
    visualize: bool,

    /// Show both min and max paths
    #[arg(long)]
    both: bool,

    /// Animate pathfinding
    #[arg(long)]
    animate: bool,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: u32,
    position: (usize, usize),
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct HexGrid {
    grid: Vec<Vec<u8>>,
    width: usize,
    height: usize,
}

impl HexGrid {
    fn new(grid: Vec<Vec<u8>>) -> Self {
        let height = grid.len();
        let width = if height > 0 { grid[0].len() } else { 0 };
        Self {
            grid,
            width,
            height,
        }
    }

    fn generate(width: usize, height: usize) -> Self {
        let mut rng = rand::thread_rng();
        let grid = (0..height)
            .map(|_| (0..width).map(|_| rng.gen_range(0..=255)).collect())
            .collect();
        Self::new(grid)
    }

    fn from_file(filename: &str) -> io::Result<Self> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut grid = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let row: Vec<u8> = line
                .split_whitespace()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect();
            if !row.is_empty() {
                grid.push(row);
            }
        }

        Ok(Self::new(grid))
    }

    fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        for row in &self.grid {
            let line: Vec<String> = row.iter().map(|&v| format!("{:02X}", v)).collect();
            writeln!(file, "{}", line.join(" "))?;
        }
        Ok(())
    }

    fn get_neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let (row, col) = pos;
        let mut neighbors = Vec::new();

        // Hex grid neighbors: up, down, left, right
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        for (dr, dc) in directions {
            let new_row = row as i32 + dr;
            let new_col = col as i32 + dc;

            if new_row >= 0
                && new_row < self.height as i32
                && new_col >= 0
                && new_col < self.width as i32
            {
                neighbors.push((new_row as usize, new_col as usize));
            }
        }

        neighbors
    }

    fn dijkstra(
        &self,
        start: (usize, usize),
        end: (usize, usize),
        animate: bool,
    ) -> Option<(Vec<(usize, usize)>, u32)> {
        let mut heap = BinaryHeap::new();
        let mut dist: HashMap<(usize, usize), u32> = HashMap::new();
        let mut prev: HashMap<(usize, usize), (usize, usize)> = HashMap::new();

        dist.insert(start, 0);
        heap.push(State {
            cost: 0,
            position: start,
        });

        let mut step = 0;

        while let Some(State { cost, position }) = heap.pop() {
            if animate {
                step += 1;
                print!(
                    "\rStep {}: Exploring ({},{}) - cost: {}",
                    step, position.0, position.1, cost
                );
                io::stdout().flush().ok();
                thread::sleep(Duration::from_millis(50));
            }

            if position == end {
                if animate {
                    println!();
                }
                let mut path = Vec::new();
                let mut current = end;
                path.push(current);

                while current != start {
                    if let Some(&p) = prev.get(&current) {
                        path.push(p);
                        current = p;
                    } else {
                        break;
                    }
                }

                path.reverse();
                return Some((path, cost));
            }

            if cost > *dist.get(&position).unwrap_or(&u32::MAX) {
                continue;
            }

            for neighbor in self.get_neighbors(position) {
                let edge_cost = self.grid[neighbor.0][neighbor.1] as u32;
                let next_cost = cost + edge_cost;

                if next_cost < *dist.get(&neighbor).unwrap_or(&u32::MAX) {
                    dist.insert(neighbor, next_cost);
                    prev.insert(neighbor, position);
                    heap.push(State {
                        cost: next_cost,
                        position: neighbor,
                    });
                }
            }
        }

        None
    }

    fn find_min_path(&self, animate: bool) -> Option<(Vec<(usize, usize)>, u32)> {
        let start = (0, 0);
        let end = (self.height - 1, self.width - 1);
        self.dijkstra(start, end, animate)
    }

    fn find_max_path(&self, animate: bool) -> Option<(Vec<(usize, usize)>, u32)> {
        // For max path, invert the costs
        let mut inverted_grid = self.grid.clone();
        for row in &mut inverted_grid {
            for val in row {
                *val = 255 - *val;
            }
        }
        let inverted = HexGrid::new(inverted_grid);
        let start = (0, 0);
        let end = (self.height - 1, self.width - 1);

        if let Some((path, _inverted_cost)) = inverted.dijkstra(start, end, animate) {
            // Calculate actual cost from original grid
            let actual_cost: u32 = path
                .iter()
                .skip(1)
                .map(|&(r, c)| self.grid[r][c] as u32)
                .sum();
            Some((path, actual_cost))
        } else {
            None
        }
    }

    fn print_grid(&self) {
        for row in &self.grid {
            for &val in row {
                print!("{:02X} ", val);
            }
            println!();
        }
    }

    fn visualize(&self, path: Option<&Vec<(usize, usize)>>, title: &str) {
        println!("\n{}:", title);
        println!("{}", "=".repeat(title.len() + 1));
        println!();

        let path_set: HashMap<(usize, usize), usize> = path
            .map(|p| p.iter().enumerate().map(|(i, &pos)| (pos, i)).collect())
            .unwrap_or_default();

        for (r, row) in self.grid.iter().enumerate() {
            for (c, &val) in row.iter().enumerate() {
                if path_set.contains_key(&(r, c)) {
                    // Path cells in bold white (for min path) or red (for max path)
                    if title.contains("MINIMUM") {
                        print!("\x1b[1;97m{:02X}\x1b[0m ", val);
                    } else if title.contains("MAXIMUM") {
                        print!("\x1b[1;91m{:02X}\x1b[0m ", val);
                    } else {
                        // Shouldn't happen, but use white as fallback
                        print!("\x1b[1;97m{:02X}\x1b[0m ", val);
                    }
                } else {
                    // Normal cells with diagonal gradient based on position
                    let color_code = Self::position_to_color(r, c, self.height, self.width);
                    print!("\x1b[{}m{:02X}\x1b[0m ", color_code, val);
                }
            }
            println!();
        }
        println!();
    }

    fn position_to_color(row: usize, col: usize, height: usize, width: usize) -> String {
        let max_sum = (height - 1) + (width - 1);
        let diagonal_sum = row + col;

        // Calculate progress from 0.0 to 1.0
        let t = if max_sum > 0 {
            diagonal_sum as f32 / max_sum as f32
        } else {
            0.0
        };

        // Map t to Hue (0 to 330 degrees) for Red -> Pink spectrum
        let hue = t * 330.0;

        // HSV to RGB conversion (S=1.0, V=1.0)
        let c = 1.0;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());

        let (r, g, b) = if hue < 60.0 {
            (c, x, 0.0)
        } else if hue < 120.0 {
            (x, c, 0.0)
        } else if hue < 180.0 {
            (0.0, c, x)
        } else if hue < 240.0 {
            (0.0, x, c)
        } else if hue < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        // Map RGB (0.0-1.0) to ANSI 6x6x6 cube (0-5)
        let r_idx = (r * 5.0).round() as u8;
        let g_idx = (g * 5.0).round() as u8;
        let b_idx = (b * 5.0).round() as u8;

        let ansi_code = 16 + 36 * r_idx + 6 * g_idx + b_idx;
        format!("38;5;{}", ansi_code)
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let grid = if let Some(size_str) = &args.generate {
        let parts: Vec<&str> = size_str.split('x').collect();
        if parts.len() != 2 {
            eprintln!("Invalid size format. Use WIDTHxHEIGHT (e.g., 8x4)");
            std::process::exit(1);
        }

        let width: usize = parts[0].parse().expect("Invalid width");
        let height: usize = parts[1].parse().expect("Invalid height");

        println!("Generating {}x{} hexadecimal grid...", width, height);
        let grid = HexGrid::generate(width, height);

        if let Some(output_file) = &args.output {
            grid.save_to_file(output_file)?;
            println!("Map saved to: {}", output_file);
        }

        println!("\nGenerated map:");
        grid.print_grid();
        println!();

        grid
    } else if let Some(map_file) = &args.map_file {
        println!("Analyzing hexadecimal grid...");
        let grid = HexGrid::from_file(map_file)?;
        println!("Grid size: {}x{}", grid.width, grid.height);
        println!("Start: (0,0) = 0x{:02X}", grid.grid[0][0]);
        println!(
            "End: ({},{}) = 0x{:02X}",
            grid.height - 1,
            grid.width - 1,
            grid.grid[grid.height - 1][grid.width - 1]
        );
        println!();
        grid
    } else {
        eprintln!("Error: Provide either a map file or use --generate");
        std::process::exit(1);
    };

    if args.animate {
        println!("Searching for minimum cost path...");
    }

    // Find minimum cost path
    if let Some((min_path, min_cost)) = grid.find_min_path(args.animate) {
        println!("MINIMUM COST PATH:");
        println!("==================");
        println!("Total cost: 0x{:X} ({} decimal)", min_cost, min_cost);
        println!("Path length: {} steps", min_path.len() - 1);
        print!("Path:\n(");
        for (i, &(r, c)) in min_path.iter().enumerate() {
            if i > 0 {
                print!("→");
            }
            print!("({},{})", r, c);
        }
        println!(")\n");

        println!("Step-by-step costs:");
        print!("Start 0x{:02X} ({},{})", grid.grid[0][0], 0, 0);
        let mut total = 0u32;
        for &(r, c) in min_path.iter().skip(1) {
            let cost = grid.grid[r][c] as u32;
            total += cost;
            println!("\n→ 0x{:02X} ({},{}) +{}", cost, r, c, cost);
        }
        println!("Total: 0x{:X} ({})", total, total);
        println!();

        if args.both {
            if args.animate {
                println!("\nSearching for maximum cost path...");
            }
            if let Some((max_path, max_cost)) = grid.find_max_path(args.animate) {
                println!("MAXIMUM COST PATH:");
                println!("==================");
                println!("Total cost: 0x{:X} ({} decimal)", max_cost, max_cost);
                println!("Path length: {} steps", max_path.len() - 1);
                print!("Path:\n(");
                for (i, &(r, c)) in max_path.iter().enumerate() {
                    if i > 0 {
                        print!("→");
                    }
                    print!("({},{})", r, c);
                }
                println!(")\n");

                println!("Step-by-step costs:");
                print!("Start 0x{:02X} ({},{})", grid.grid[0][0], 0, 0);
                let mut total = 0u32;
                for &(r, c) in max_path.iter().skip(1) {
                    let cost = grid.grid[r][c] as u32;
                    total += cost;
                    println!("\n→ 0x{:02X} ({},{}) +{}", cost, r, c, cost);
                }
                println!("Total: 0x{:X} ({})", total, total);
                println!();

                if args.visualize {
                    // Show three separate grids
                    grid.visualize(None, "HEXADECIMAL GRID (rainbow gradient)");
                    grid.visualize(Some(&min_path), "MINIMUM COST PATH (shown in WHITE)");
                    println!("Cost: {} (minimum)\\n", min_cost);
                    grid.visualize(Some(&max_path), "MAXIMUM COST PATH (shown in RED)");
                    println!("Cost: {} (maximum)\\n", max_cost);
                }
            }
        } else if args.visualize {
            grid.visualize(Some(&min_path), "HEXADECIMAL GRID (rainbow gradient)");
            println!("Cost: {} (minimum)", min_cost);
        }
    } else {
        println!("No path found!");
    }

    Ok(())
}
