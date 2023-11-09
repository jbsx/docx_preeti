use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct Map {
    character_map: HashMap<String, String>,
    post_rules: Vec<Vec<String>>,
}

pub fn convert(input: String) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(std::env::current_dir()?.join("src").join("preeti.json"));
    let map_string = fs::read_to_string(&file_path).unwrap();
    let rules: Map = serde_json::from_str(&map_string)?;

    let mut res = String::new();

    for i in input.split("") {
        res.push_str(rules.character_map.get(i).unwrap_or(&String::new()));
    }

    for i in rules.post_rules {
        res = res.replace(&i[0], &i[1]);
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_test() {
        let test_string = "ckg ;dfhs] sf]gf 3/d] a]6Ls] hgd x]nf;] ck;u'0f cf/ 3[0ff s/n hfo x}o . PsfO{;f} ztfAbLd] rn /xn ;dfh Pvlgof] 3/d] klxnsf af/] a]6f hGd e]nfk/ ef]h et]/ dgfjn hfo x} . jx] hux j]6Ls] hGd e]nf;] …3/d] nIdLÚ s] cfudg xf]on x}o sxn hfo x} n]lsg dgd] j]6f xj}s] OR5f bjfs] 5f]6df]6 ef]het]/ cf/ ljwL ljwfg s/n hfo x} . ha ls O ;dfhs] yfxf gxo gf/L lx O &gt;[i6Ls] gf/L xL rfnj x}o .".to_string();
        let converted_string = "अपन समाजके कोना घरमे बेटीके जनम हेलासे अपसगुण आर घृणा करल जाय हैय । एकाईसौ शताब्दीमे चल रहल समाज एखनियो घरमे पहिलका बारे बेटा जन्म भेलापर भोज भतेर मनावल जाय है । वहे जगह वेटीके जन्म भेलासे ‘घरमे लक्ष्मी’ के आगमन होयल हैय कहल जाय है लेकिन मनमे वेटा हवैके इच्छा दवाके छोटमोट भोजभतेर आर विधी विधान करल जाय है । जब कि इ समाजके थाहा नहय नारी हि इ ७नतसृष्टीके नारी ही चालव हैय ।".to_string();

        assert_eq!(convert(test_string).unwrap(), converted_string);
    }
}
