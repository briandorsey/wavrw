use nom::bytes::streaming::{tag, take};
use nom::error::ErrorKind;
use nom::multi::many0;
use nom::number::streaming::le_u32;
use nom::Err::Error;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct Wav<'a> {
    pub riff_size: u32,
    pub chunks: Vec<UnknownChunk<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct UnknownChunk<'a> {
    pub chunk_id: &'a [u8],
    pub chunk_size: u32,
    //pub data: Vec<u8>,
}

pub fn parse(input: &[u8]) -> IResult<&[u8], Wav> {
    let (input, riff_size) = parse_riff_header(input)?;
    let (input, chunks) = parse_chunks(input)?;

    Ok((
        input,
        Wav {
            riff_size,
            chunks: chunks,
        },
    ))
}

fn parse_riff_header(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, _) = tag("RIFF")(input)?;
    let (input, riff_size) = le_u32(input)?;
    let (input, _) = tag("WAVE")(input)?;
    Ok((input, riff_size))
}

fn parse_chunks(input: &[u8]) -> IResult<&[u8], Vec<UnknownChunk>> {
    let (input, chunks) = many0(parse_chunk)(input)?;
    Ok((input, chunks))
}

fn parse_chunk(input: &[u8]) -> IResult<&[u8], UnknownChunk> {
    // TODO: figure out the right way to deal with size mismatches
    if input.len() < 4 {
        // seems like there must be a cleaner way to abort a parser on error?
        return Err(Error(nom::error::Error {
            input: input,
            code: ErrorKind::Many0,
        }));
    }
    let (input, chunk_id) = take(4_usize)(input)?;
    let (input, chunk_size) = le_u32(input)?;
    let (input, _) = take(chunk_size)(input)?;
    Ok((
        input,
        UnknownChunk {
            chunk_id: chunk_id,
            chunk_size: chunk_size,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::decode;
    use hexdump::hexdump;

    #[test]
    fn riff_header() {
        // RIFF 2398 WAVE
        let header: &[u8] = &decode("524946465E09000057415645").unwrap();
        println!("{header:?}");
        let (_, actual) = parse(header).unwrap();
        assert_eq!(
            Wav {
                riff_size: 2398,
                chunks: vec![],
            },
            actual
        );
    }

    #[test]
    fn parse_chunk_length() {
        let tests = [(
            &decode("666D7420 10000000 01000100 80BB0000 80320200 03001800".replace(' ', ""))
                .unwrap(),
            UnknownChunk {
                chunk_id: "fmt ".as_bytes().try_into().unwrap(),
                chunk_size: 16,
            },
            &[] as &[u8],
        )];
        for (input, expected_output, expected_remaining_input) in tests {
            hexdump(input);
            let (remaining_input, output) = parse_chunk(input).unwrap();
            assert_eq!(expected_output, output);
            assert_eq!(expected_remaining_input, remaining_input);
        }
    }

    #[test]
    fn parse_header_fmt() {
        let data = &mut decode("52494646 5E090000 57415645".replace(' ', "")).unwrap();

        data.extend(
            &decode("666D7420 10000000 01000100 80BB0000 80320200 03001800".replace(' ', ""))
                .unwrap(),
        );
        let tests = [(
            data,
            Wav {
                riff_size: 2398,
                chunks: vec![UnknownChunk {
                    chunk_id: "fmt ".as_bytes().try_into().unwrap(),
                    chunk_size: 16,
                }],
            },
            &[] as &[u8],
        )];
        for (input, expected_output, expected_remaining_input) in tests {
            hexdump(input);
            let (remaining_input, output) = parse(input).unwrap();
            assert_eq!(expected_output, output);
            hexdump(remaining_input);
            assert_eq!(expected_remaining_input, remaining_input);
        }
    }

    #[test]
    fn decode_spaces() {
        let a = &decode("666D7420 10000000 01000100 80BB0000 80320200 03001800".replace(' ', ""))
            .unwrap();
        let b = &decode("666D7420100000000100010080BB00008032020003001800").unwrap();
        assert_eq!(a, b);
    }
}
