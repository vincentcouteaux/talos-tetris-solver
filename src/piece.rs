use crate::bitmap::Bitmap2D;
use std::collections::HashMap;

pub struct PieceVariant {
    pub bitmap: Bitmap2D,
    //pub origin: (u32, u32),
    pub origin: (usize, usize),
}

pub struct Piece {
    pub variants: Vec<PieceVariant>
}

pub const PIECE_ORDER: [char ; 7] = ['J', 'I', 'L', 'T', 'S', 'Z', 'O'];

pub fn get_standard_pieces() -> HashMap<char, Piece> {
    let mut pieces_by_name = HashMap::new();

    pieces_by_name.insert('J', Piece { variants:
            vec![PieceVariant { bitmap: Bitmap2D { shape: (3, 2), data: vec![0b010111 << 58] }, origin: (0,1) },
                 PieceVariant { bitmap: Bitmap2D { shape: (3, 2), data: vec![0b111010 << 58] }, origin: (0,0) },
                 PieceVariant { bitmap: Bitmap2D { shape: (2, 3), data: vec![0b100111 << 58] }, origin: (0,0) },
                 PieceVariant { bitmap: Bitmap2D { shape: (2, 3), data: vec![0b111001 << 58] }, origin: (0,0) }] });

    pieces_by_name.insert('I', Piece { variants:
            vec![PieceVariant { bitmap: Bitmap2D { shape: (1, 4), data: vec![0b1111 << 60] }, origin: (0,0) },
                 PieceVariant { bitmap: Bitmap2D { shape: (4, 1), data: vec![0b1111 << 60] }, origin: (0,0) }] });

    pieces_by_name.insert('L', Piece { variants:
            vec![PieceVariant { bitmap: Bitmap2D { shape: (3, 2), data: vec![0b101011 << 58] }, origin: (0,0) },
                 PieceVariant { bitmap: Bitmap2D { shape: (3, 2), data: vec![0b110101 << 58] }, origin: (0,0) },
                 PieceVariant { bitmap: Bitmap2D { shape: (2, 3), data: vec![0b001111 << 58] }, origin: (0,2) },
                 PieceVariant { bitmap: Bitmap2D { shape: (2, 3), data: vec![0b111100 << 58] }, origin: (0,0) }] });

    pieces_by_name.insert('T', Piece { variants:
            vec![PieceVariant { bitmap: Bitmap2D { shape: (3, 2), data: vec![0b011101 << 58] }, origin: (0,1) },
                 PieceVariant { bitmap: Bitmap2D { shape: (3, 2), data: vec![0b101110 << 58] }, origin: (0,0) },
                 PieceVariant { bitmap: Bitmap2D { shape: (2, 3), data: vec![0b010111 << 58] }, origin: (0,1) },
                 PieceVariant { bitmap: Bitmap2D { shape: (2, 3), data: vec![0b111010 << 58] }, origin: (0,0) }] });

    pieces_by_name.insert('S', Piece { variants:
            vec![PieceVariant { bitmap: Bitmap2D { shape: (3, 2), data: vec![0b101101 << 58] }, origin: (0,0) },
                 PieceVariant { bitmap: Bitmap2D { shape: (2, 3), data: vec![0b011110 << 58] }, origin: (0,1) }] });

    pieces_by_name.insert('Z', Piece { variants:
            vec![PieceVariant { bitmap: Bitmap2D { shape: (3, 2), data: vec![0b011110 << 58] }, origin: (0,1) },
                 PieceVariant { bitmap: Bitmap2D { shape: (2, 3), data: vec![0b110011 << 58] }, origin: (0,0) }] });

    pieces_by_name.insert('O', Piece { variants:
            vec![PieceVariant { bitmap: Bitmap2D { shape: (2, 2), data: vec![0b1111 << 60] }, origin: (0,0) }]});

    pieces_by_name
}

pub fn get_padded_pieces(board_size: (usize, usize)) -> Vec<HashMap<(usize, usize), Vec<Bitmap2D>>> {
    let pieces_by_name = get_standard_pieces();
    let mut out = Vec::with_capacity(7);
    for piece_letter in PIECE_ORDER {
        let mut position_dic: HashMap<(usize, usize), Vec<Bitmap2D>> =
            HashMap::new();
        let piece = pieces_by_name.get(&piece_letter).unwrap();
        for variant in piece.variants.iter() {
            for offset_x in 0..(board_size.0 - variant.bitmap.shape.0 + 1) {
                for offset_y in 0..(board_size.1 - variant.bitmap.shape.1 + 1) {
                    let padded = variant.bitmap.pad_to(board_size,
                                                       (offset_x, offset_y));
                    let new_origin = (offset_x + variant.origin.0,
                                      offset_y + variant.origin.1);
                    match position_dic.get_mut(&new_origin) {
                        Some(vec) => vec.push(padded),
                        None => { position_dic.insert(new_origin, vec![padded]); },
                    }
                }
            }
        }
        out.push(position_dic);
    }
    out
}

// algorithme :
// R = un tableau de longueur 7 avec le nombre de pièces restantes de chaque type <- copié à chaque
// appel récursif (voir s'il existe immutable dict ???)
// piece_versions = pour chaque pièce, une hashmap <position, iterateur sur les versions paddées de la
// pièce> <-- constant sur tout l'algo
//
// def fill_board(board, R, position):
//   if board[position] -> fill_board(board, R, position+1) 
//   for piece_id in R if R[piece_id] > 0
//      for piece_version in piece_versions[piece_id][position]
//          if fits(piece_version, board):
//              rclone = R.clone()
//              rclone[piece_id] -= 1
//              new_board = board `or` piece_version
//              fill_board(new_board, rclone, position+1)
// ajouter une condition de succès ou d'échec
// ajouter un moyen de récupérer la solution
