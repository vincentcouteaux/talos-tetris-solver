mod bitmap;
mod piece;

use bitmap::Bitmap2D;
use piece::*;
use std::collections::HashMap;

fn increment(shape: (usize, usize), index: (usize, usize)) -> Option<(usize, usize)> {
    let flat_index = index.0*shape.1 + index.1 + 1;
    if shape.1 == 0 { return None }
    let new_index = (flat_index / shape.1,
                     flat_index % shape.1);
    if new_index.0 >= shape.0 { None } else { Some(new_index) }
}

fn fill_board(board: &Bitmap2D, remaining_pieces: [u32; 7],
              position: (usize, usize),
              padded_pieces: &Vec<HashMap<(usize, usize), Vec<Bitmap2D>>>) -> bool {
    let next_pos = match increment(board.shape, position) {
        Some(coord) => coord,
        None => return true
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
                    if fill_board(&new_board, new_remaining, next_pos, padded_pieces) {
                        println!("{}: {} {}", PIECE_ORDER[piece_id],
                                 position.0, position.1);
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn main() {
    let board_size = (4,4);
    let board = Bitmap2D { shape: board_size, data: vec![0] };
    let piece_count: [u32; 7] = [0, 0, 0, 0, 0, 0, 4];

    let pieces = get_padded_pieces(board_size);

    println!("{}", fill_board(&board, piece_count, (0, 0), &pieces));

    //let j_piece = Bitmap2D { shape: (3, 2), data: vec![0b010111 << 58] };
    //println!("{}", j_piece.pad_to((10, 10), (2, 3)).to_string());
    //println!("{}", j_piece.pad_to((20, 20), (2, 3)).to_string());
    //println!("{}", j_piece.pad_to((4, 4), (1, 2)).to_string());

    //let pieces = get_standard_pieces();
    //for (name, piece) in pieces {
    //    println!("{name} piece");
    //    for variant in piece.variants {
    //        println!("{}", variant.bitmap.to_string());
    //        println!("    ");
    //    }
    //}

    //for piece_dic in pieces {
    //    println!("######");
    //    for ((posx, posy), variants) in piece_dic {
    //        println!("**** ({posx}, {posy})");
    //        for variant in variants {
    //            println!("{}", variant.to_string());
    //            println!("--");
    //        }
    //    }
    //}
    //println!("{pieces:?}");
}

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
