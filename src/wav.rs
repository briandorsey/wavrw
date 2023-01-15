use nom::bytes::streaming::tag;
use nom::number::streaming::le_u32;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct Wav {
    pub riff_size: u32,
}

pub fn parse(input: &[u8]) -> IResult<&[u8], Wav> {
    let (input, _) = tag("RIFF")(input)?;
    let (input, riff_size) = le_u32(input)?;

    Ok((input, Wav { riff_size }))
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::decode;

    #[test]
    fn riff_header() {
        // RIFF 2398
        let header: &[u8] = &decode("524946465E090000").unwrap();
        println!("{header:?}");
        assert_eq!(Wav { riff_size: 2398 }, parse(header).unwrap().1);
    }
}
