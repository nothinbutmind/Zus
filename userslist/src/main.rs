use rs_merkle::{MerkleTree, algorithms::Sha256};
use rs_merkle::Hasher;


fn main() {
    let addys = ["0x","1x","2x","3x" , "4x"];
    //println!("{:?}", addys[0].as_bytes());
    let leaves:Vec<[u8;32]> = addys.iter().map(|x| Sha256::hash(x.as_bytes())).collect();

    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    //println!("{:?}", merkle_tree.root());
    match merkle_tree.root(){
        Some(root) =>{
            let hex = root 
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<String>();
            println!("root hex {}" , hex);
    }
    None => {
        println!("Fuck u");
    }

    }
}

fn belongsTo(root: String) -> {
    
}
