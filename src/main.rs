use obentou_cli::app::App;

use anyhow::Result;

fn main() -> Result<()> {
    let mut app = App::new(std::env::args().nth(1).unwrap_or_default())?;
    app.run()?;
    Ok(())
}

/*
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_arrange() {

        write_info!("test arrange");
        
        // let cards = vec!["Link-2x2", "Map-2x4", "Counter-1x4", "Link-2x4",
           //              "Section-1x8", "Note-2x2", "Album-4x4"];

        let string_array = [
            "Section-1x8", "Note-4x4", "Note-4x2", "Note-2x4", "Social-2x2", "Counter-1x4", "Section-1x8",
            "Social-2x2", "Social-2x4", "Link-1x4", "Link-2x4", "Album-4x4", "Section-1x8", "Photo-4x2",
            "Section-1x8",
        ];

        let cards: Vec<String> = string_array.iter().map(|s| String::from(*s)).collect();


        let l = arrange_grid((50, 8), &cards);
        write_info!(format!("len: {}", l.len()));
        for c in l.iter() {
            write_info!(format!("{:?}", c));
        }


    // assert_eq!(adder(-2, 3), 1);
    }
}
*/
