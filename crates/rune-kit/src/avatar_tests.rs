use super::*;

#[test]
fn initials_take_the_first_and_last_words() {
    assert_eq!(initials("Ada Lovelace"), "AL");
    assert_eq!(initials("Grace Brewster Murray Hopper"), "GH");
    assert_eq!(initials("plato"), "P");
    assert_eq!(initials("  "), "");
    assert_eq!(initials("(bot) éclair"), "BÉ");
}

#[test]
fn a_name_keeps_its_colour() {
    assert_eq!(name_color("Ada"), name_color("ada "));
    let names = ["Ada", "Grace", "Alan", "Edsger", "Barbara", "Donald", "Frances", "Ken", "Margaret", "Tim"];
    let mut hexes: Vec<String> = names.iter().map(|n| name_color(n).hex()).collect();
    hexes.sort();
    hexes.dedup();
    assert!(hexes.len() >= 6, "names spread over the hues: {hexes:?}");
}
