mod bitmap;
mod piece;

use bitmap::Bitmap2D;
use piece::*;
use std::collections::HashMap;
use std::env;
use std::error::Error;

fn increment(shape: (usize, usize), index: (usize, usize)) -> Option<(usize, usize)> {
    let flat_index = index.0*shape.1 + index.1 + 1;
    if shape.1 == 0 { return None }
    let new_index = (flat_index / shape.1,
                     flat_index % shape.1);
    if new_index.0 >= shape.0 { None } else { Some(new_index) }
}

fn fill_board<'a, 'b>(board: &'b Bitmap2D, remaining_pieces: [u32; 7],
              position: (usize, usize),
              padded_pieces: &'a Vec<HashMap<(usize, usize), Vec<Bitmap2D>>>)
                -> Option<Vec<&'a Bitmap2D>> {
    let next_pos = match increment(board.shape, position) {
        Some(coord) => coord,
        None => return Some(vec![]) // not necessarily true ?
    };
    if board.get(position).unwrap_or(false)  {
        return fill_board(board, remaining_pieces, next_pos, padded_pieces);
    }
    for (piece_id, piece_dict) in padded_pieces.iter().enumerate() {
        if remaining_pieces[piece_id] == 0 { continue }
        if let Some(variants) = piece_dict.get(&position) {
            for variant in variants {
                if !board.intersects(variant) {
                    let new_board = board.or(variant);
                    let mut new_remaining = remaining_pieces.clone();
                    new_remaining[piece_id] -= 1;
                    if let Some(mut solution) = fill_board(&new_board, new_remaining,
                                                       next_pos, padded_pieces) {
                        solution.push(&variant);
                        return Some(solution);
                    }
                }
            }
        }
    }
    None
}

const USAGE_MSG: &str = "Usage: W H PIECES\nExample: 5 8 IIIIJJLLSZ";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next();
    let board_size = (args.next().ok_or(USAGE_MSG)?
                          .parse::<usize>()
                          .map_err(|_| USAGE_MSG)?,
                      args.next().ok_or(USAGE_MSG)?
                          .parse::<usize>()
                          .map_err(|_| USAGE_MSG)?);

    let board = Bitmap2D { shape: board_size, data: vec![0] };

    let mut piece_ids: HashMap<char, usize> = HashMap::new();
    for (idx, piece_name) in PIECE_ORDER.iter().enumerate() {
        piece_ids.insert(*piece_name, idx);
    }

    let mut piece_count: [u32; 7] = [0 ; 7];
    let pieces_str = args.next().ok_or(USAGE_MSG)?;
    for piece_name in pieces_str.chars() {
        piece_count[*piece_ids.get(&piece_name)
            .ok_or(format!("Unrecognized piece name: {}", piece_name))?] += 1;
    }

    let pieces = get_padded_pieces(board_size);
    let solution = fill_board(&board, piece_count, (0, 0), &pieces);

    match solution {
        Some(sol) => {
            println!("Solution:\n{}",
                     to_ansi(Bitmap2D::print_all(sol.into_iter())));
        },
        None => println!("No solution")
    }
    Ok (())
}

fn to_ansi(ipt_str: String) -> String {
    format!("{}\x1b[0m\n",
        ipt_str.chars().map(|x| {
            let color = match u32::from_str_radix(&x.to_string(), 16) {
                Err(_) => return x.to_string(),
                Ok(col) => col
            };
            let code = if color < 8 { 40 + color } else { 92 + color };
            format!("\x1b[{code}m  ")

        }).collect::<Vec<String>>().join("").replace("\n", "\x1b[0m\n"))
}
// TODO add test cases for fill_board
// Return all solutions (with Vec<Vec<...>> instead of Option<Vec>..
// Use multithreading to parallelize the search
// Manage cases where n_pieces != 4*H*W

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        assert_eq!(increment((4, 4), (2, 2)), Some((2,3)));
        assert_eq!(increment((4, 4), (2, 3)), Some((3,0)));
        assert_eq!(increment((4, 4), (3, 3)), None);
        assert_eq!(increment((0, 0), (3, 3)), None);
    }
}
