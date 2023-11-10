use htmlescape::decode_html;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct Map {
    character_map: HashMap<String, String>,
    post_rules: Vec<Vec<String>>,
}

pub fn preeti_to_unicode(input: String) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(std::env::current_dir()?.join("src").join("preeti.json"));
    let map_string = fs::read_to_string(&file_path).unwrap();
    let rules: Map = serde_json::from_str(&map_string)?;

    //normalise html entities
    let normalised_input = decode_html(&input).unwrap_or(input);

    //convert
    let mut res = String::new();
    for i in normalised_input.split("") {
        res.push_str(rules.character_map.get(i).unwrap_or(&i.to_owned()));
    }

    for i in rules.post_rules {
        res = res.replace(&i[0], &i[1]);
    }

    let mut res_utf16: Vec<u16> = res.encode_utf16().collect();

    // fixing \u093f placement
    let mut idx = 0;
    while idx < res_utf16.len() - 1 {
        if res_utf16[idx] == 2367 {
            res_utf16.swap(idx, idx + 1);
            idx += 2;
        } else {
            idx += 1;
        }
    }

    Ok(String::from_utf16_lossy(&res_utf16))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preeti_to_unicode() {
        let test_string = "klxrfg".to_owned();
        let control: String = "पहिचान".to_owned();

        let converted = preeti_to_unicode(test_string).unwrap();

        assert_eq!(converted, control);
    }

    #[test]
    fn test_html_entity() {
        let test_string = "ckg ;dfhs] sf]gf 3/d] a]6Ls] hgd x]nf;] ck;u'0f cf/ 3[0ff s/n hfo x}o . PsfO{;f} ztfAbLd] rn /xn ;dfh Pvlgof] 3/d] klxnsf af/] a]6f hGd e]nfk/ ef]h et]/ dgfjn hfo x} . jx] hux j]6Ls] hGd e]nf;] …3/d] nIdLÚ s] cfudg xf]on x}o sxn hfo x} n]lsg dgd] j]6f xj}s] OR5f bjfs] 5f]6df]6 ef]het]/ cf/ ljwL ljwfg s/n hfo x} . ha ls O ;dfhs] yfxf gxo gf/L lx O &gt;[i6Ls] gf/L xL rfnj x}o . ".to_string();
        let control = "अपन समाजके कोना घरमे बेटीके जनम हेलासे अपसगुण आर घृणा करल जाय हैय । एकाईसौ शताब्दीमे चल रहल समाज एखनियो घरमे पहिलका बारे बेटा जन्म भेलापर भोज भतेर मनावल जाय है । वहे जगह वेटीके जन्म भेलासे ‘घरमे लक्ष्मी’ के आगमन होयल हैय कहल जाय है लेकिन मनमे वेटा हवैके इच्छा दवाके छोटमोट भोजभतेर आर विधी विधान करल जाय है । जब कि इ समाजके थाहा नहय नारी हि इ श्रृष्टीके नारी ही चालव हैय । ".to_string();

        let converted = preeti_to_unicode(test_string).unwrap();

        //println!(
        //    "\n\n\n{:?}\n\n{:?}\n\n\n",
        //    converted.encode_utf16().collect::<Vec<u16>>(),
        //    control.encode_utf16().collect::<Vec<u16>>()
        //);

        assert_eq!(converted, control);
    }
}
