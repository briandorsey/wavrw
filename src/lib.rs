use binrw::io::TakeSeekExt;
use binrw::Endian;
use binrw::NullString;
use binrw::{binrw, helpers, io::SeekFrom, BinRead, BinResult, BinWrite, Error, PosValue};
use itertools::Itertools;
use std::cmp::min;
use std::fmt::{Debug, Display, Formatter};
use std::io::BufReader;
use std::io::{Read, Seek};
use std::str::FromStr;

// helper types
// ----

pub const fn fourcc(id: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*id)
}

pub trait KnownChunkID {
    const ID: FourCC;
}

pub trait ChunkID {
    fn id(&self) -> FourCC;
}

pub trait SizedChunk: ChunkID {
    /// Returns the logical (used) size in bytes of the chunk.
    fn size(&self) -> u32;
}

pub trait Summarizable {
    /// Returns a short text summary of the contents of the chunk.
    fn summary(&self) -> String;

    /// Returns an iterator over a sequence of contents of the contained
    /// chunk as strings (field, value).
    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(std::iter::empty())
    }
}

pub trait Chunk: SizedChunk + Summarizable + Debug {}

impl<T> ChunkID for T
where
    T: KnownChunkID,
{
    fn id(&self) -> FourCC {
        T::ID
    }
}

// Helper for cleaner Result.map() calls when boxing inner chunk.
// Reduces code repetition, but mostly helps the compiler with type
// inference.
fn box_chunk<T: Chunk + 'static>(t: T) -> Box<dyn Chunk> {
    Box::new(t)
}

#[binrw]
#[brw(big)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FourCC(pub [u8; 4]);

impl Display for FourCC {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", String::from_utf8_lossy(&self.0),)?;
        Ok(())
    }
}

impl Debug for FourCC {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "FourCC({}=", String::from_utf8_lossy(&self.0),)?;
        write!(f, "{:?})", &self.0)?;
        Ok(())
    }
}

// needed for assert in br() attribute
impl<'a> PartialEq<&'a FourCC> for FourCC {
    fn eq(&self, other: &&'a FourCC) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<FourCC> for &'a FourCC {
    fn eq(&self, other: &FourCC) -> bool {
        *self == other
    }
}

#[derive(Debug)]
struct FixedStrErr;

#[derive(BinWrite, PartialEq, Eq)]
/// FixedStr holds Null terminated fixed length strings (from BEXT for example)
///
/// FixedStr is intended to be used via binrw's [BinRead] trait and its
/// Null parsing is implmented there. Do not directly create the struct
/// or that logic will be bypassed. If there is a future need, we should
/// implement a ::new() constructor which in turn calls the [FixedStr::read_options()]
/// implementation.
struct FixedStr<const N: usize>([u8; N]);

// FixedStr display design question. RIFF spec uses ""Z notation for fixed strings. Should we do the same?

impl<const N: usize> Debug for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "FixedStr<{}>(\"{}\")", N, self)
    }
}

impl<const N: usize> Display for FixedStr<N> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            String::from_utf8_lossy(&self.0).trim_end_matches('\0')
        )
    }
}

impl<const N: usize> FromStr for FixedStr<N> {
    type Err = FixedStrErr;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        let mut array_tmp = [0u8; N];
        let l = min(s.len(), N);
        array_tmp[..l].copy_from_slice(&s.as_bytes()[..l]);
        Ok(FixedStr::<N>(array_tmp))
    }
}

impl<const N: usize> BinRead for FixedStr<N> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut values: [u8; N] = [0; N];
        let mut index = 0;

        loop {
            if index >= N {
                return Ok(Self(values));
            }
            let val = <u8>::read_options(reader, endian, ())?;
            if val == 0 {
                let offset = (N - index - 1).try_into();
                return match offset {
                    Ok(offset) => {
                        reader.seek(SeekFrom::Current(offset))?;
                        Ok(Self(values))
                    }
                    Err(err) => Err(Error::Custom {
                        pos: index as u64,
                        err: Box::new(err),
                    }),
                };
            }
            values[index] = val;
            index += 1;
        }
    }
}

// parsing helpers
// ----

/// `metadata_chunks` parses a WAV file chunk by chunk, continuing
///  even if some chunks have parsing errors.
pub fn metadata_chunks<R>(reader: R) -> Result<Vec<BinResult<Box<dyn Chunk>>>, std::io::Error>
where
    R: Read + Seek,
{
    let mut reader = BufReader::new(reader);

    // TODO: research errors and figure out an error plan for wavrw
    // remove wrapping Result, and map IO and BinErrors to wavrw errors
    let riff = RiffChunk::read(&mut reader).map_err(std::io::Error::other)?;
    // TODO: convert assert into returned wav error type
    assert_eq!(
        riff.form_type,
        FourCC(*b"WAVE"),
        "{} != WAVE",
        riff.form_type
    );

    let mut buff: [u8; 4] = [0; 4];
    let mut offset = reader.stream_position()?;
    let mut chunks: Vec<BinResult<Box<dyn Chunk>>> = Vec::new();

    loop {
        let chunk_id = {
            reader.read_exact(&mut buff)?;
            buff
        };
        let chunk_size = {
            reader.read_exact(&mut buff)?;
            u32::from_le_bytes(buff)
        };

        reader.seek(SeekFrom::Current(-8))?;
        // TODO: convert to match on each specific chunk type: fewer seeks and better error messages
        let res = ChunkEnum::read(&mut reader).map(box_chunk);
        chunks.push(res);

        // setup for next iteration
        offset += chunk_size as u64 + 8;
        // RIFF offsets must be on word boundaries (divisible by 2)
        if offset % 2 == 1 {
            offset += 1;
        };
        if offset != reader.stream_position()? {
            // TODO: inject error into chunk vec and remove print
            println!(
                "WARNING: {}: parsed less data than chunk size",
                FourCC(chunk_id)
            );
            reader.seek(SeekFrom::Start(offset))?;
        }

        if offset >= riff.size as u64 {
            break;
        };
    }
    Ok(chunks)
}

// parsing structs
// ----

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
// const generics don't support array types yet, so let's just encode it into a u32
/// This chunk structure is a helper so the user can choose to just read a single chunk
pub struct KnownChunk<
    T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + KnownChunkID,
> {
    #[br(temp, assert(id == T::ID))]
    #[bw(calc = T::ID)]
    id: FourCC,

    // TODO: calc by querying content + extra_bytes.len() when writing, or seeking back after you know
    size: u32,

    #[br(temp)]
    #[bw(ignore)]
    begin_pos: PosValue<()>,
    // ensure that we don't read outside the bounds for this chunk
    #[br(map_stream = |r| r.take_seek(size as u64))]
    data: T,

    // assert for better error message if too many bytes processed
    // seems like it should be impossible, but found a case where T
    // used align_after to pad bytes.
    #[br(temp, assert((end_pos.pos - begin_pos.pos) <= size as u64, "(end_pos.pos - begin_pos.pos) <= size while parsing {}. end: {}, begin: {}, size: {}", T::ID, end_pos.pos, begin_pos.pos, size))]
    #[bw(ignore)]
    end_pos: PosValue<()>,

    // calculate how much was read, and read any extra bytes that remain in the chunk
    #[br(align_after = 2, count = size as u64 - (end_pos.pos - begin_pos.pos))]
    extra_bytes: Vec<u8>,
}

impl<T> KnownChunkID for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + KnownChunkID,
{
    const ID: FourCC = T::ID;
}

impl<T> SizedChunk for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = ()> + for<'a> BinWrite<Args<'a> = ()> + KnownChunkID,
{
    fn size(&self) -> u32 {
        self.size + self.extra_bytes.len() as u32
    }
}

impl<T> Summarizable for KnownChunk<T>
where
    T: for<'a> BinRead<Args<'a> = ()>
        + for<'a> BinWrite<Args<'a> = ()>
        + KnownChunkID
        + Summarizable,
{
    fn summary(&self) -> String {
        self.data.summary()
    }
    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        self.data.items()
    }
}

// based on https://mediaarea.net/BWFMetaEdit/md5
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct Md5ChunkData {
    md5: u128,
}

impl KnownChunkID for Md5ChunkData {
    const ID: FourCC = FourCC(*b"MD5 ");
}

impl SizedChunk for Md5ChunkData {
    fn size(&self) -> u32 {
        16
    }
}

impl Summarizable for Md5ChunkData {
    fn summary(&self) -> String {
        format!("0x{:X}", self.md5)
    }
}

type Md5Chunk = KnownChunk<Md5ChunkData>;

impl Chunk for Md5Chunk {}

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
// http://www.tactilemedia.com/info/MCI_Control_Info.html
pub struct RiffChunk {
    pub id: FourCC,
    pub size: u32,
    pub form_type: FourCC,
}

#[allow(dead_code)]
#[binrw]
#[brw(little, repr = u16)]
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatTag {
    Unknown = 0x0000,
    Pcm = 0x0001,
    Adpcm = 0x0002,
    IeeeFloat = 0x0003,
    Vselp = 0x0004,
    IbmCvsd = 0x0005,
    Alaw = 0x0006,
    Mulaw = 0x0007,
    Dts = 0x0008,
    Drm = 0x0009,
    Wmavoice9 = 0x000A,
    Wmavoice10 = 0x000B,
    OkiAdpcm = 0x0010,
    DviAdpcm = 0x0011,
    MediaspaceAdpcm = 0x0012,
    SierraAdpcm = 0x0013,
    G723Adpcm = 0x0014,
    Digistd = 0x0015,
    Digifix = 0x0016,
    DialogicOkiAdpcm = 0x0017,
    MediavisionAdpcm = 0x0018,
    CuCodec = 0x0019,
    HpDynVoice = 0x001A,
    YamahaAdpcm = 0x0020,
    Sonarc = 0x0021,
    DspgroupTruespeech = 0x0022,
    Echosc1 = 0x0023,
    AudiofileAf36 = 0x0024,
    Aptx = 0x0025,
    AudiofileAf10 = 0x0026,
    Prosody1612 = 0x0027,
    Lrc = 0x0028,
    DolbyAc2 = 0x0030,
    Gsm610x31 = 0x0031,
    Msnaudio = 0x0032,
    AntexAdpcme = 0x0033,
    ControlResVqlpc = 0x0034,
    Digireal = 0x0035,
    Digiadpcm = 0x0036,
    ControlResCr10 = 0x0037,
    NmsVbxadpcm = 0x0038,
    CsImaadpcm = 0x0039,
    Echosc3 = 0x003A,
    RockwellAdpcm = 0x003B,
    RockwellDigitalk = 0x003C,
    Xebec = 0x003D,
    G721Adpcm = 0x0040,
    G728Celp = 0x0041,
    Msg723 = 0x0042,
    IntelG7231 = 0x0043,
    IntelG729 = 0x0044,
    SharpG726 = 0x0045,
    Mpeg = 0x0050,
    Rt24 = 0x0052,
    Pac = 0x0053,
    Mpeglayer3 = 0x0055,
    LucentG723 = 0x0059,
    Cirrus = 0x0060,
    Espcm = 0x0061,
    Voxware = 0x0062,
    CanopusAtrac = 0x0063,
    ApicomG726Adpcm = 0x0064,
    ApicomG722Adpcm = 0x0065,
    Dsat = 0x0066,
    DsatDisplay = 0x0067,
    VoxwareByteAligned = 0x0069,
    VoxwareAc8 = 0x0070,
    VoxwareAc10 = 0x0071,
    VoxwareAc16 = 0x0072,
    VoxwareAc20 = 0x0073,
    VoxwareRt24 = 0x0074,
    VoxwareRt29 = 0x0075,
    VoxwareRt29Hw = 0x0076,
    VoxwareVr12 = 0x0077,
    VoxwareVr18 = 0x0078,
    VoxwareTq40 = 0x0079,
    VoxwareSc3 = 0x007A,
    VoxwareSc31 = 0x007B,
    Softsound = 0x0080,
    VoxwareTq60 = 0x0081,
    Msrt24 = 0x0082,
    G729A = 0x0083,
    MviMvi2 = 0x0084,
    DfG726 = 0x0085,
    DfGsm610 = 0x0086,
    Isiaudio = 0x0088,
    Onlive = 0x0089,
    MultitudeFtSx20 = 0x008A,
    InfocomItsG721Adpcm = 0x008B,
    ConvediaG729 = 0x008C,
    Congruency = 0x008D,
    Sbc24 = 0x0091,
    DolbyAc3Spdif = 0x0092,
    MediasonicG723 = 0x0093,
    Prosody8Kbps = 0x0094,
    ZyxelAdpcm = 0x0097,
    PhilipsLpcbb = 0x0098,
    Packed = 0x0099,
    MaldenPhonytalk = 0x00A0,
    RacalRecorderGsm = 0x00A1,
    RacalRecorderG720A = 0x00A2,
    RacalRecorderG7231 = 0x00A3,
    RacalRecorderTetraAcelp = 0x00A4,
    NecAac = 0x00B0,
    RawAac1 = 0x00FF,
    RhetorexAdpcm = 0x0100,
    Irat = 0x0101,
    VivoG723 = 0x0111,
    VivoSiren = 0x0112,
    PhilipsCelp = 0x0120,
    PhilipsGrundig = 0x0121,
    DigitalG723 = 0x0123,
    SanyoLdAdpcm = 0x0125,
    SiprolabAceplnet = 0x0130,
    SiprolabAcelp4800 = 0x0131,
    SiprolabAcelp8V3 = 0x0132,
    SiprolabG729 = 0x0133,
    SiprolabG729A = 0x0134,
    SiprolabKelvin = 0x0135,
    VoiceageAmr = 0x0136,
    DictaphoneG726Adpcm = 0x0140,
    DictaphoneCelp68 = 0x0141,
    DictaphoneCelp54 = 0x0142,
    QualcommPurevoice = 0x0150,
    QualcommHalfrate = 0x0151,
    Tubgsm = 0x0155,
    Msaudio1 = 0x0160,
    Wmaudio2 = 0x0161,
    Wmaudio3 = 0x0162,
    WmaudioLossless = 0x0163,
    Wmaspdif = 0x0164,
    UnisysNapAdpcm = 0x0170,
    UnisysNapUlaw = 0x0171,
    UnisysNapAlaw = 0x0172,
    UnisysNap16K = 0x0173,
    SycomAcmSyc008 = 0x0174,
    SycomAcmSyc701G726L = 0x0175,
    SycomAcmSyc701Celp54 = 0x0176,
    SycomAcmSyc701Celp68 = 0x0177,
    KnowledgeAdventureAdpcm = 0x0178,
    FraunhoferIisMpeg2Aac = 0x0180,
    DtsDs = 0x0190,
    CreativeAdpcm = 0x0200,
    CreativeFastspeech8 = 0x0202,
    CreativeFastspeech10 = 0x0203,
    UherAdpcm = 0x0210,
    UleadDvAudio = 0x0215,
    UleadDvAudio1 = 0x0216,
    Quarterdeck = 0x0220,
    IlinkVc = 0x0230,
    RawSport = 0x0240,
    EsstAc3 = 0x0241,
    GenericPassthru = 0x0249,
    IpiHsx = 0x0250,
    IpiRpelp = 0x0251,
    Cs2 = 0x0260,
    SonyScx = 0x0270,
    SonyScy = 0x0271,
    SonyAtrac3 = 0x0272,
    SonySpc = 0x0273,
    TelumAudio = 0x0280,
    TelumIaAudio = 0x0281,
    NorcomVoiceSystemsAdpcm = 0x0285,
    FmTownsSnd = 0x0300,
    Micronas = 0x0350,
    MicronasCelp833 = 0x0351,
    BtvDigital = 0x0400,
    IntelMusicCoder = 0x0401,
    IndeoAudio = 0x0402,
    QdesignMusic = 0x0450,
    On2Vp7Audio = 0x0500,
    On2Vp6Audio = 0x0501,
    VmeVmpcm = 0x0680,
    Tpc = 0x0681,
    LightwaveLossless = 0x08AE,
    Oligsm = 0x1000,
    Oliadpcm = 0x1001,
    Olicelp = 0x1002,
    Olisbc = 0x1003,
    Oliopr = 0x1004,
    LhCodec = 0x1100,
    LhCodecCelp = 0x1101,
    LhCodecSbc8 = 0x1102,
    LhCodecSbc12 = 0x1103,
    LhCodecSbc16 = 0x1104,
    Norris = 0x1400,
    Isiaudio2 = 0x1401,
    SoundspaceMusicompress = 0x1500,
    MpegAdtsAac = 0x1600,
    MpegRawAac = 0x1601,
    MpegLoas = 0x1602,
    NokiaMpegAdtsAac = 0x1608,
    NokiaMpegRawAac = 0x1609,
    VodafoneMpegAdtsAac = 0x160A,
    VodafoneMpegRawAac = 0x160B,
    MpegHeaac = 0x1610,
    VoxwareRt24Speech = 0x181C,
    SonicfoundryLossless = 0x1971,
    InningsTelecomAdpcm = 0x1979,
    LucentSx8300P = 0x1C07,
    LucentSx5363S = 0x1C0C,
    Cuseeme = 0x1F03,
    NtcsoftAlf2CmAcm = 0x1FC4,
    Dvm = 0x2000,
    Dts2 = 0x2001,
    Makeavis = 0x3313,
    DivioMpeg4Aac = 0x4143,
    NokiaAdaptiveMultirate = 0x4201,
    DivioG726 = 0x4243,
    LeadSpeech = 0x434C,
    LeadVorbis = 0x564C,
    WavpackAudio = 0x5756,
    Alac = 0x6C61,
    OggVorbisMode1 = 0x674F,
    OggVorbisMode2 = 0x6750,
    OggVorbisMode3 = 0x6751,
    OggVorbisMode1Plus = 0x676F,
    OggVorbisMode2Plus = 0x6770,
    OggVorbisMode3Plus = 0x6771,
    ThreeComNbx = 0x7000,
    Opus = 0x704F,
    FaadAac = 0x706D,
    AmrNb = 0x7361,
    AmrWb = 0x7362,
    AmrWp = 0x7363,
    GsmAmrCbr = 0x7A21,
    GsmAmrVbrSid = 0x7A22,
    ComverseInfosysG7231 = 0xA100,
    ComverseInfosysAvqsbc = 0xA101,
    ComverseInfosysSbc = 0xA102,
    SymbolG729A = 0xA103,
    VoiceageAmrWb = 0xA104,
    IngenientG726 = 0xA105,
    Mpeg4Aac = 0xA106,
    EncoreG726 = 0xA107,
    ZollAsao = 0xA108,
    SpeexVoice = 0xA109,
    VianixMasc = 0xA10A,
    Wm9SpectrumAnalyzer = 0xA10B,
    WmfSpectrumAnayzer = 0xA10C,
    Gsm610 = 0xA10D,
    Gsm620 = 0xA10E,
    Gsm660 = 0xA10F,
    Gsm690 = 0xA110,
    GsmAdaptiveMultirateWb = 0xA111,
    PolycomG722 = 0xA112,
    PolycomG728 = 0xA113,
    PolycomG729A = 0xA114,
    PolycomSiren = 0xA115,
    GlobalIpIlbc = 0xA116,
    RadiotimeTimeShiftRadio = 0xA117,
    NiceAca = 0xA118,
    NiceAdpcm = 0xA119,
    VocordG721 = 0xA11A,
    VocordG726 = 0xA11B,
    VocordG7221 = 0xA11C,
    VocordG728 = 0xA11D,
    VocordG729 = 0xA11E,
    VocordG729A = 0xA11F,
    VocordG7231 = 0xA120,
    VocordLbc = 0xA121,
    NiceG728 = 0xA122,
    FraceTelecomG729 = 0xA123,
    Codian = 0xA124,
    DolbyAc4 = 0xAC40,
    Flac = 0xF1AC,
    Extensible = 0xFFFE,
    Development = 0xFFFF,
}

impl Display for FormatTag {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        use FormatTag::*;
        let output = match self {
            Unknown => "WAVE_FORMAT_UNKNOWN",
            Pcm => "WAVE_FORMAT_PCM",
            Adpcm => "WAVE_FORMAT_ADPCM",
            IeeeFloat => "WAVE_FORMAT_IEEE_FLOAT",
            Vselp => "WAVE_FORMAT_VSELP",
            IbmCvsd => "WAVE_FORMAT_IBM_CVSD",
            Alaw => "WAVE_FORMAT_ALAW",
            Mulaw => "WAVE_FORMAT_MULAW",
            Dts => "WAVE_FORMAT_DTS",
            Drm => "WAVE_FORMAT_DRM",
            Wmavoice9 => "WAVE_FORMAT_WMAVOICE9",
            Wmavoice10 => "WAVE_FORMAT_WMAVOICE10",
            OkiAdpcm => "WAVE_FORMAT_OKI_ADPCM",
            DviAdpcm => "WAVE_FORMAT_DVI_ADPCM",
            MediaspaceAdpcm => "WAVE_FORMAT_MEDIASPACE_ADPCM",
            SierraAdpcm => "WAVE_FORMAT_SIERRA_ADPCM",
            G723Adpcm => "WAVE_FORMAT_G723_ADPCM",
            Digistd => "WAVE_FORMAT_DIGISTD",
            Digifix => "WAVE_FORMAT_DIGIFIX",
            DialogicOkiAdpcm => "WAVE_FORMAT_DIALOGIC_OKI_ADPCM",
            MediavisionAdpcm => "WAVE_FORMAT_MEDIAVISION_ADPCM",
            CuCodec => "WAVE_FORMAT_CU_CODEC",
            HpDynVoice => "WAVE_FORMAT_HP_DYN_VOICE",
            YamahaAdpcm => "WAVE_FORMAT_YAMAHA_ADPCM",
            Sonarc => "WAVE_FORMAT_SONARC",
            DspgroupTruespeech => "WAVE_FORMAT_DSPGROUP_TRUESPEECH",
            Echosc1 => "WAVE_FORMAT_ECHOSC1",
            AudiofileAf36 => "WAVE_FORMAT_AUDIOFILE_AF36",
            Aptx => "WAVE_FORMAT_APTX",
            AudiofileAf10 => "WAVE_FORMAT_AUDIOFILE_AF10",
            Prosody1612 => "WAVE_FORMAT_PROSODY_1612",
            Lrc => "WAVE_FORMAT_LRC",
            DolbyAc2 => "WAVE_FORMAT_DOLBY_AC2",
            Gsm610x31 => "WAVE_FORMAT_GSM610",
            Msnaudio => "WAVE_FORMAT_MSNAUDIO",
            AntexAdpcme => "WAVE_FORMAT_ANTEX_ADPCME",
            ControlResVqlpc => "WAVE_FORMAT_CONTROL_RES_VQLPC",
            Digireal => "WAVE_FORMAT_DIGIREAL",
            Digiadpcm => "WAVE_FORMAT_DIGIADPCM",
            ControlResCr10 => "WAVE_FORMAT_CONTROL_RES_CR10",
            NmsVbxadpcm => "WAVE_FORMAT_NMS_VBXADPCM",
            CsImaadpcm => "WAVE_FORMAT_CS_IMAADPCM",
            Echosc3 => "WAVE_FORMAT_ECHOSC3",
            RockwellAdpcm => "WAVE_FORMAT_ROCKWELL_ADPCM",
            RockwellDigitalk => "WAVE_FORMAT_ROCKWELL_DIGITALK",
            Xebec => "WAVE_FORMAT_XEBEC",
            G721Adpcm => "WAVE_FORMAT_G721_ADPCM",
            G728Celp => "WAVE_FORMAT_G728_CELP",
            Msg723 => "WAVE_FORMAT_MSG723",
            IntelG7231 => "WAVE_FORMAT_INTEL_G723_1",
            IntelG729 => "WAVE_FORMAT_INTEL_G729",
            SharpG726 => "WAVE_FORMAT_SHARP_G726",
            Mpeg => "WAVE_FORMAT_MPEG",
            Rt24 => "WAVE_FORMAT_RT24",
            Pac => "WAVE_FORMAT_PAC",
            Mpeglayer3 => "WAVE_FORMAT_MPEGLAYER3",
            LucentG723 => "WAVE_FORMAT_LUCENT_G723",
            Cirrus => "WAVE_FORMAT_CIRRUS",
            Espcm => "WAVE_FORMAT_ESPCM",
            Voxware => "WAVE_FORMAT_VOXWARE",
            CanopusAtrac => "WAVE_FORMAT_CANOPUS_ATRAC",
            ApicomG726Adpcm => "WAVE_FORMAT_G726_ADPCM",
            ApicomG722Adpcm => "WAVE_FORMAT_G722_ADPCM",
            Dsat => "WAVE_FORMAT_DSAT",
            DsatDisplay => "WAVE_FORMAT_DSAT_DISPLAY",
            VoxwareByteAligned => "WAVE_FORMAT_VOXWARE_BYTE_ALIGNED",
            VoxwareAc8 => "WAVE_FORMAT_VOXWARE_AC8",
            VoxwareAc10 => "WAVE_FORMAT_VOXWARE_AC10",
            VoxwareAc16 => "WAVE_FORMAT_VOXWARE_AC16",
            VoxwareAc20 => "WAVE_FORMAT_VOXWARE_AC20",
            VoxwareRt24 => "WAVE_FORMAT_VOXWARE_RT24",
            VoxwareRt29 => "WAVE_FORMAT_VOXWARE_RT29",
            VoxwareRt29Hw => "WAVE_FORMAT_VOXWARE_RT29HW",
            VoxwareVr12 => "WAVE_FORMAT_VOXWARE_VR12",
            VoxwareVr18 => "WAVE_FORMAT_VOXWARE_VR18",
            VoxwareTq40 => "WAVE_FORMAT_VOXWARE_TQ40",
            VoxwareSc3 => "WAVE_FORMAT_VOXWARE_SC3",
            VoxwareSc31 => "WAVE_FORMAT_VOXWARE_SC3_1",
            Softsound => "WAVE_FORMAT_SOFTSOUND",
            VoxwareTq60 => "WAVE_FORMAT_VOXWARE_TQ60",
            Msrt24 => "WAVE_FORMAT_MSRT24",
            G729A => "WAVE_FORMAT_G729A",
            MviMvi2 => "WAVE_FORMAT_MVI_MVI2",
            DfG726 => "WAVE_FORMAT_DF_G726",
            DfGsm610 => "WAVE_FORMAT_DF_GSM610",
            Isiaudio => "WAVE_FORMAT_ISIAUDIO",
            Onlive => "WAVE_FORMAT_ONLIVE",
            MultitudeFtSx20 => "WAVE_FORMAT_MULTITUDE_FT_SX20",
            InfocomItsG721Adpcm => "WAVE_FORMAT_INFOCOM_ITS_G721_ADPCM",
            ConvediaG729 => "WAVE_FORMAT_CONVEDIA_G729",
            Congruency => "WAVE_FORMAT_CONGRUENCY",
            Sbc24 => "WAVE_FORMAT_SBC24",
            DolbyAc3Spdif => "WAVE_FORMAT_DOLBY_AC3_SPDIF",
            MediasonicG723 => "WAVE_FORMAT_MEDIASONIC_G723",
            Prosody8Kbps => "WAVE_FORMAT_PROSODY_8KBPS",
            ZyxelAdpcm => "WAVE_FORMAT_ZYXEL_ADPCM",
            PhilipsLpcbb => "WAVE_FORMAT_PHILIPS_LPCBB",
            Packed => "WAVE_FORMAT_PACKED",
            MaldenPhonytalk => "WAVE_FORMAT_MALDEN_PHONYTALK",
            RacalRecorderGsm => "WAVE_FORMAT_RACAL_RECORDER_GSM",
            RacalRecorderG720A => "WAVE_FORMAT_RACAL_RECORDER_G720_A",
            RacalRecorderG7231 => "WAVE_FORMAT_RACAL_RECORDER_G723_1",
            RacalRecorderTetraAcelp => "WAVE_FORMAT_RACAL_RECORDER_TETRA_ACELP",
            NecAac => "WAVE_FORMAT_NEC_AAC",
            RawAac1 => "WAVE_FORMAT_RAW_AAC1",
            RhetorexAdpcm => "WAVE_FORMAT_RHETOREX_ADPCM",
            Irat => "WAVE_FORMAT_IRAT",
            VivoG723 => "WAVE_FORMAT_VIVO_G723",
            VivoSiren => "WAVE_FORMAT_VIVO_SIREN",
            PhilipsCelp => "WAVE_FORMAT_PHILIPS_CELP",
            PhilipsGrundig => "WAVE_FORMAT_PHILIPS_GRUNDIG",
            DigitalG723 => "WAVE_FORMAT_DIGITAL_G723",
            SanyoLdAdpcm => "WAVE_FORMAT_SANYO_LD_ADPCM",
            SiprolabAceplnet => "WAVE_FORMAT_SIPROLAB_ACEPLNET",
            SiprolabAcelp4800 => "WAVE_FORMAT_SIPROLAB_ACELP4800",
            SiprolabAcelp8V3 => "WAVE_FORMAT_SIPROLAB_ACELP8V3",
            SiprolabG729 => "WAVE_FORMAT_SIPROLAB_G729",
            SiprolabG729A => "WAVE_FORMAT_SIPROLAB_G729A",
            SiprolabKelvin => "WAVE_FORMAT_SIPROLAB_KELVIN",
            VoiceageAmr => "WAVE_FORMAT_VOICEAGE_AMR",
            DictaphoneG726Adpcm => "WAVE_FORMAT_G726ADPCM",
            DictaphoneCelp68 => "WAVE_FORMAT_DICTAPHONE_CELP68",
            DictaphoneCelp54 => "WAVE_FORMAT_DICTAPHONE_CELP54",
            QualcommPurevoice => "WAVE_FORMAT_QUALCOMM_PUREVOICE",
            QualcommHalfrate => "WAVE_FORMAT_QUALCOMM_HALFRATE",
            Tubgsm => "WAVE_FORMAT_TUBGSM",
            Msaudio1 => "WAVE_FORMAT_MSAUDIO1",
            Wmaudio2 => "WAVE_FORMAT_WMAUDIO2",
            Wmaudio3 => "WAVE_FORMAT_WMAUDIO3",
            WmaudioLossless => "WAVE_FORMAT_WMAUDIO_LOSSLESS",
            Wmaspdif => "WAVE_FORMAT_WMASPDIF",
            UnisysNapAdpcm => "WAVE_FORMAT_UNISYS_NAP_ADPCM",
            UnisysNapUlaw => "WAVE_FORMAT_UNISYS_NAP_ULAW",
            UnisysNapAlaw => "WAVE_FORMAT_UNISYS_NAP_ALAW",
            UnisysNap16K => "WAVE_FORMAT_UNISYS_NAP_16K",
            SycomAcmSyc008 => "WAVE_FORMAT_SYCOM_ACM_SYC008",
            SycomAcmSyc701G726L => "WAVE_FORMAT_SYCOM_ACM_SYC701_G726L",
            SycomAcmSyc701Celp54 => "WAVE_FORMAT_SYCOM_ACM_SYC701_CELP54",
            SycomAcmSyc701Celp68 => "WAVE_FORMAT_SYCOM_ACM_SYC701_CELP68",
            KnowledgeAdventureAdpcm => "WAVE_FORMAT_KNOWLEDGE_ADVENTURE_ADPCM",
            FraunhoferIisMpeg2Aac => "WAVE_FORMAT_FRAUNHOFER_IIS_MPEG2_AAC",
            DtsDs => "WAVE_FORMAT_DTS_DS",
            CreativeAdpcm => "WAVE_FORMAT_CREATIVE_ADPCM",
            CreativeFastspeech8 => "WAVE_FORMAT_CREATIVE_FASTSPEECH8",
            CreativeFastspeech10 => "WAVE_FORMAT_CREATIVE_FASTSPEECH10",
            UherAdpcm => "WAVE_FORMAT_UHER_ADPCM",
            UleadDvAudio => "WAVE_FORMAT_ULEAD_DV_AUDIO",
            UleadDvAudio1 => "WAVE_FORMAT_ULEAD_DV_AUDIO_1",
            Quarterdeck => "WAVE_FORMAT_QUARTERDECK",
            IlinkVc => "WAVE_FORMAT_ILINK_VC",
            RawSport => "WAVE_FORMAT_RAW_SPORT",
            EsstAc3 => "WAVE_FORMAT_ESST_AC3",
            GenericPassthru => "WAVE_FORMAT_GENERIC_PASSTHRU",
            IpiHsx => "WAVE_FORMAT_IPI_HSX",
            IpiRpelp => "WAVE_FORMAT_IPI_RPELP",
            Cs2 => "WAVE_FORMAT_CS2",
            SonyScx => "WAVE_FORMAT_SONY_SCX",
            SonyScy => "WAVE_FORMAT_SONY_SCY",
            SonyAtrac3 => "WAVE_FORMAT_SONY_ATRAC3",
            SonySpc => "WAVE_FORMAT_SONY_SPC",
            TelumAudio => "WAVE_FORMAT_TELUM_AUDIO",
            TelumIaAudio => "WAVE_FORMAT_TELUM_IA_AUDIO",
            NorcomVoiceSystemsAdpcm => "WAVE_FORMAT_NORCOM_VOICE_SYSTEMS_ADPCM",
            FmTownsSnd => "WAVE_FORMAT_FM_TOWNS_SND",
            Micronas => "WAVE_FORMAT_MICRONAS",
            MicronasCelp833 => "WAVE_FORMAT_MICRONAS_CELP833",
            BtvDigital => "WAVE_FORMAT_BTV_DIGITAL",
            IntelMusicCoder => "WAVE_FORMAT_INTEL_MUSIC_CODER",
            IndeoAudio => "WAVE_FORMAT_INDEO_AUDIO",
            QdesignMusic => "WAVE_FORMAT_QDESIGN_MUSIC",
            On2Vp7Audio => "WAVE_FORMAT_ON2_VP7_AUDIO",
            On2Vp6Audio => "WAVE_FORMAT_ON2_VP6_AUDIO",
            VmeVmpcm => "WAVE_FORMAT_VME_VMPCM",
            Tpc => "WAVE_FORMAT_TPC",
            LightwaveLossless => "WAVE_FORMAT_LIGHTWAVE_LOSSLESS",
            Oligsm => "WAVE_FORMAT_OLIGSM",
            Oliadpcm => "WAVE_FORMAT_OLIADPCM",
            Olicelp => "WAVE_FORMAT_OLICELP",
            Olisbc => "WAVE_FORMAT_OLISBC",
            Oliopr => "WAVE_FORMAT_OLIOPR",
            LhCodec => "WAVE_FORMAT_LH_CODEC",
            LhCodecCelp => "WAVE_FORMAT_LH_CODEC_CELP",
            LhCodecSbc8 => "WAVE_FORMAT_LH_CODEC_SBC8",
            LhCodecSbc12 => "WAVE_FORMAT_LH_CODEC_SBC12",
            LhCodecSbc16 => "WAVE_FORMAT_LH_CODEC_SBC16",
            Norris => "WAVE_FORMAT_NORRIS",
            Isiaudio2 => "WAVE_FORMAT_ISIAUDIO_2",
            SoundspaceMusicompress => "WAVE_FORMAT_SOUNDSPACE_MUSICOMPRESS",
            MpegAdtsAac => "WAVE_FORMAT_MPEG_ADTS_AAC",
            MpegRawAac => "WAVE_FORMAT_MPEG_RAW_AAC",
            MpegLoas => "WAVE_FORMAT_MPEG_LOAS",
            NokiaMpegAdtsAac => "WAVE_FORMAT_NOKIA_MPEG_ADTS_AAC",
            NokiaMpegRawAac => "WAVE_FORMAT_NOKIA_MPEG_RAW_AAC",
            VodafoneMpegAdtsAac => "WAVE_FORMAT_VODAFONE_MPEG_ADTS_AAC",
            VodafoneMpegRawAac => "WAVE_FORMAT_VODAFONE_MPEG_RAW_AAC",
            MpegHeaac => "WAVE_FORMAT_MPEG_HEAAC",
            VoxwareRt24Speech => "WAVE_FORMAT_VOXWARE_RT24_SPEECH",
            SonicfoundryLossless => "WAVE_FORMAT_SONICFOUNDRY_LOSSLESS",
            InningsTelecomAdpcm => "WAVE_FORMAT_INNINGS_TELECOM_ADPCM",
            LucentSx8300P => "WAVE_FORMAT_LUCENT_SX8300P",
            LucentSx5363S => "WAVE_FORMAT_LUCENT_SX5363S",
            Cuseeme => "WAVE_FORMAT_CUSEEME",
            NtcsoftAlf2CmAcm => "WAVE_FORMAT_NTCSOFT_ALF2CM_ACM",
            Dvm => "WAVE_FORMAT_DVM",
            Dts2 => "WAVE_FORMAT_DTS2",
            Makeavis => "WAVE_FORMAT_MAKEAVIS",
            DivioMpeg4Aac => "WAVE_FORMAT_DIVIO_MPEG4_AAC",
            NokiaAdaptiveMultirate => "WAVE_FORMAT_NOKIA_ADAPTIVE_MULTIRATE",
            DivioG726 => "WAVE_FORMAT_DIVIO_G726",
            LeadSpeech => "WAVE_FORMAT_LEAD_SPEECH",
            LeadVorbis => "WAVE_FORMAT_LEAD_VORBIS",
            WavpackAudio => "WAVE_FORMAT_WAVPACK_AUDIO",
            Alac => "WAVE_FORMAT_ALAC",
            OggVorbisMode1 => "WAVE_FORMAT_OGG_VORBIS_MODE_1",
            OggVorbisMode2 => "WAVE_FORMAT_OGG_VORBIS_MODE_2",
            OggVorbisMode3 => "WAVE_FORMAT_OGG_VORBIS_MODE_3",
            OggVorbisMode1Plus => "WAVE_FORMAT_OGG_VORBIS_MODE_1_PLUS",
            OggVorbisMode2Plus => "WAVE_FORMAT_OGG_VORBIS_MODE_2_PLUS",
            OggVorbisMode3Plus => "WAVE_FORMAT_OGG_VORBIS_MODE_3_PLUS",
            ThreeComNbx => "WAVE_FORMAT_3COM_NBX",
            Opus => "WAVE_FORMAT_OPUS",
            FaadAac => "WAVE_FORMAT_FAAD_AAC",
            AmrNb => "WAVE_FORMAT_AMR_NB",
            AmrWb => "WAVE_FORMAT_AMR_WB",
            AmrWp => "WAVE_FORMAT_AMR_WP",
            GsmAmrCbr => "WAVE_FORMAT_GSM_AMR_CBR",
            GsmAmrVbrSid => "WAVE_FORMAT_GSM_AMR_VBR_SID",
            ComverseInfosysG7231 => "WAVE_FORMAT_COMVERSE_INFOSYS_G723_1",
            ComverseInfosysAvqsbc => "WAVE_FORMAT_COMVERSE_INFOSYS_AVQSBC",
            ComverseInfosysSbc => "WAVE_FORMAT_COMVERSE_INFOSYS_SBC",
            SymbolG729A => "WAVE_FORMAT_SYMBOL_G729_A",
            VoiceageAmrWb => "WAVE_FORMAT_VOICEAGE_AMR_WB",
            IngenientG726 => "WAVE_FORMAT_INGENIENT_G726",
            Mpeg4Aac => "WAVE_FORMAT_MPEG4_AAC",
            EncoreG726 => "WAVE_FORMAT_ENCORE_G726",
            ZollAsao => "WAVE_FORMAT_ZOLL_ASAO",
            SpeexVoice => "WAVE_FORMAT_SPEEX_VOICE",
            VianixMasc => "WAVE_FORMAT_VIANIX_MASC",
            Wm9SpectrumAnalyzer => "WAVE_FORMAT_WM9_SPECTRUM_ANALYZER",
            WmfSpectrumAnayzer => "WAVE_FORMAT_WMF_SPECTRUM_ANAYZER",
            Gsm610 => "WAVE_FORMAT_GSM_610",
            Gsm620 => "WAVE_FORMAT_GSM_620",
            Gsm660 => "WAVE_FORMAT_GSM_660",
            Gsm690 => "WAVE_FORMAT_GSM_690",
            GsmAdaptiveMultirateWb => "WAVE_FORMAT_GSM_ADAPTIVE_MULTIRATE_WB",
            PolycomG722 => "WAVE_FORMAT_POLYCOM_G722",
            PolycomG728 => "WAVE_FORMAT_POLYCOM_G728",
            PolycomG729A => "WAVE_FORMAT_POLYCOM_G729_A",
            PolycomSiren => "WAVE_FORMAT_POLYCOM_SIREN",
            GlobalIpIlbc => "WAVE_FORMAT_GLOBAL_IP_ILBC",
            RadiotimeTimeShiftRadio => "WAVE_FORMAT_RADIOTIME_TIME_SHIFT_RADIO",
            NiceAca => "WAVE_FORMAT_NICE_ACA",
            NiceAdpcm => "WAVE_FORMAT_NICE_ADPCM",
            VocordG721 => "WAVE_FORMAT_VOCORD_G721",
            VocordG726 => "WAVE_FORMAT_VOCORD_G726",
            VocordG7221 => "WAVE_FORMAT_VOCORD_G722_1",
            VocordG728 => "WAVE_FORMAT_VOCORD_G728",
            VocordG729 => "WAVE_FORMAT_VOCORD_G729",
            VocordG729A => "WAVE_FORMAT_VOCORD_G729_A",
            VocordG7231 => "WAVE_FORMAT_VOCORD_G723_1",
            VocordLbc => "WAVE_FORMAT_VOCORD_LBC",
            NiceG728 => "WAVE_FORMAT_NICE_G728",
            FraceTelecomG729 => "WAVE_FORMAT_FRACE_TELECOM_G729",
            Codian => "WAVE_FORMAT_CODIAN",
            DolbyAc4 => "WAVE_FORMAT_DOLBY_AC4",
            Flac => "WAVE_FORMAT_FLAC",
            Extensible => "WAVE_FORMAT_EXTENSIBLE",
            Development => "WAVE_FORMAT_DEVELOPMENT",
        };
        write!(f, "{} (0x{:x})", output, *self as u16)?;
        Ok(())
    }
}

// based on http://soundfile.sapp.org/doc/WaveFormat/
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct FmtChunkData {
    format_tag: FormatTag,
    channels: u16,
    samples_per_sec: u32,
    avg_bytes_per_sec: u32,
    block_align: u16,
    bits_per_sample: u16,
}
// TODO: properly handle different fmt chunk additions from later specs

impl KnownChunkID for FmtChunkData {
    const ID: FourCC = FourCC(*b"fmt ");
}

type FmtChunk = KnownChunk<FmtChunkData>;

impl FmtChunk {
    pub fn summary(&self) -> String {
        format!(
            "{} chan, {}/{}",
            self.data.channels,
            self.data.bits_per_sample,
            self.data.samples_per_sec,
            // TODO: format_tag
        )
    }

    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }
}

// Iteration based on pattern from https://stackoverflow.com/questions/30218886/how-to-implement-iterator-and-intoiterator-for-a-simple-struct

impl<'a> IntoIterator for &'a FmtChunk {
    type Item = (String, String);
    type IntoIter = FmtChunkIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FmtChunkIterator {
            fmt: self,
            index: 0,
        }
    }
}

pub struct FmtChunkIterator<'a> {
    fmt: &'a FmtChunk,
    index: usize,
}

impl<'a> Iterator for FmtChunkIterator<'a> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some((
                "format_tag".to_string(),
                self.fmt.data.format_tag.to_string(),
            )),
            2 => Some(("channels".to_string(), self.fmt.data.channels.to_string())),
            3 => Some((
                "samples_per_sec".to_string(),
                self.fmt.data.samples_per_sec.to_string(),
            )),
            4 => Some((
                "avg_bytes_per_sec".to_string(),
                self.fmt.data.avg_bytes_per_sec.to_string(),
            )),
            5 => Some((
                "block_align".to_string(),
                self.fmt.data.block_align.to_string(),
            )),
            6 => Some((
                "bits_per_sample".to_string(),
                self.fmt.data.bits_per_sample.to_string(),
            )),
            _ => None,
        }
    }
}

/// `data` chunk parser which skips all audio data
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct DataChunkData {}

impl KnownChunkID for DataChunkData {
    const ID: FourCC = FourCC(*b"data");
}

type DataChunk = KnownChunk<DataChunkData>;

impl DataChunk {
    pub fn summary(&self) -> String {
        "audio data".to_string()
    }
}

#[binrw]
#[br(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct ListInfoChunkData {
    #[brw(assert(list_type == FourCC(*b"INFO")))]
    list_type: FourCC,
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    chunks: Vec<InfoChunkEnum>,
}

impl KnownChunkID for ListInfoChunkData {
    const ID: FourCC = FourCC(*b"LIST");
}

type ListInfoChunk = KnownChunk<ListInfoChunkData>;

impl ListInfoChunk {
    pub fn summary(&self) -> String {
        format!(
            "{}: {}",
            self.data.list_type,
            self.data.chunks.iter().map(|c| c.id()).join(", ")
        )
    }

    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(
            self.data
                .chunks
                .iter()
                .map(|c| (c.id().to_string(), c.value())),
        )
    }
}

/// InfoChunkData is a genericised container for LIST INFO chunks
///
/// ```
/// # use wavrw::IcmtChunkData;
/// let icmt = IcmtChunkData::new("comment");
/// ```
///
/// # Examples:
///
/// Since const generics do not support arrays, it's storing the FourCC
/// id as a `u32`... which makes instantiation awkward. There is a helper
/// function [fourcc] to make it a bit easier in the general case:
///
/// ```
/// # use wavrw::{InfoChunkData, fourcc};
/// # use binrw::NullString;
/// let icmt =  InfoChunkData::<{ fourcc(b"ICMT") }> {
///     value: NullString("comment".into()),
/// };
/// ```
///
/// and for all of the INFO types from the original 1991 WAV spec, there
/// is an additional alias:
///
/// ```
/// # use wavrw::IcmtChunkData;
/// # use binrw::NullString;
/// let icmt = IcmtChunkData {
///     value: NullString("comment".into()),
/// };
/// ```
#[binrw]
#[br(little)]
#[derive(PartialEq, Eq)]
pub struct InfoChunkData<const I: u32> {
    pub value: NullString,
}

impl<const I: u32> KnownChunkID for InfoChunkData<I> {
    const ID: FourCC = FourCC(I.to_le_bytes());
}

impl<const I: u32> Debug for InfoChunkData<I> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "InfoChunkData<{}> {{ value: {:?} }}",
            String::from_utf8_lossy(I.to_le_bytes().as_slice()),
            self.value,
        )?;
        Ok(())
    }
}

impl<const I: u32> Summarizable for InfoChunkData<I> {
    fn summary(&self) -> String {
        self.value.to_string()
    }
}

impl<const I: u32> InfoChunkData<I> {
    pub fn new(value: &str) -> Self {
        InfoChunkData::<I> {
            value: value.into(),
        }
    }
}

pub type IarlChunkData = InfoChunkData<{ fourcc(b"IARL") }>;
pub type IgnrChunkData = InfoChunkData<{ fourcc(b"IGNR") }>;
pub type IkeyChunkData = InfoChunkData<{ fourcc(b"IKEY") }>;
pub type IlgtChunkData = InfoChunkData<{ fourcc(b"ILGT") }>;
pub type ImedChunkData = InfoChunkData<{ fourcc(b"IMED") }>;
pub type InamChunkData = InfoChunkData<{ fourcc(b"INAM") }>;
pub type IpltChunkData = InfoChunkData<{ fourcc(b"IPLT") }>;
pub type IprdChunkData = InfoChunkData<{ fourcc(b"IPRD") }>;
pub type IsbjChunkData = InfoChunkData<{ fourcc(b"ISBJ") }>;
pub type IsftChunkData = InfoChunkData<{ fourcc(b"ISFT") }>;
pub type IshpChunkData = InfoChunkData<{ fourcc(b"ISHP") }>;
pub type IartChunkData = InfoChunkData<{ fourcc(b"IART") }>;
pub type IsrcChunkData = InfoChunkData<{ fourcc(b"ISRC") }>;
pub type IsrfChunkData = InfoChunkData<{ fourcc(b"ISRF") }>;
pub type ItchChunkData = InfoChunkData<{ fourcc(b"ITCH") }>;
pub type IcmsChunkData = InfoChunkData<{ fourcc(b"ICMS") }>;
pub type IcmtChunkData = InfoChunkData<{ fourcc(b"ICMT") }>;
pub type IcopChunkData = InfoChunkData<{ fourcc(b"ICOP") }>;
pub type IcrdChunkData = InfoChunkData<{ fourcc(b"ICRD") }>;
pub type IcrpChunkData = InfoChunkData<{ fourcc(b"ICRP") }>;
pub type IdpiChunkData = InfoChunkData<{ fourcc(b"IDPI") }>;
pub type IengChunkData = InfoChunkData<{ fourcc(b"IENG") }>;

pub type IarlChunk = KnownChunk<IarlChunkData>;
pub type IgnrChunk = KnownChunk<IgnrChunkData>;
pub type IkeyChunk = KnownChunk<IkeyChunkData>;
pub type IlgtChunk = KnownChunk<IlgtChunkData>;
pub type ImedChunk = KnownChunk<ImedChunkData>;
pub type InamChunk = KnownChunk<InamChunkData>;
pub type IpltChunk = KnownChunk<IpltChunkData>;
pub type IprdChunk = KnownChunk<IprdChunkData>;
pub type IsbjChunk = KnownChunk<IsbjChunkData>;
pub type IsftChunk = KnownChunk<IsftChunkData>;
pub type IshpChunk = KnownChunk<IshpChunkData>;
pub type IartChunk = KnownChunk<IartChunkData>;
pub type IsrcChunk = KnownChunk<IsrcChunkData>;
pub type IsrfChunk = KnownChunk<IsrfChunkData>;
pub type ItchChunk = KnownChunk<ItchChunkData>;
pub type IcmsChunk = KnownChunk<IcmsChunkData>;
pub type IcmtChunk = KnownChunk<IcmtChunkData>;
pub type IcopChunk = KnownChunk<IcopChunkData>;
pub type IcrdChunk = KnownChunk<IcrdChunkData>;
pub type IcrpChunk = KnownChunk<IcrpChunkData>;
pub type IdpiChunk = KnownChunk<IdpiChunkData>;
pub type IengChunk = KnownChunk<IengChunkData>;

impl Chunk for IarlChunk {}
impl Chunk for IgnrChunk {}
impl Chunk for IkeyChunk {}
impl Chunk for IlgtChunk {}
impl Chunk for ImedChunk {}
impl Chunk for InamChunk {}
impl Chunk for IpltChunk {}
impl Chunk for IprdChunk {}
impl Chunk for IsbjChunk {}
impl Chunk for IsftChunk {}
impl Chunk for IshpChunk {}
impl Chunk for IartChunk {}
impl Chunk for IsrcChunk {}
impl Chunk for IsrfChunk {}
impl Chunk for ItchChunk {}
impl Chunk for IcmsChunk {}
impl Chunk for IcmtChunk {}
impl Chunk for IcopChunk {}
impl Chunk for IcrdChunk {}
impl Chunk for IcrpChunk {}
impl Chunk for IdpiChunk {}
impl Chunk for IengChunk {}

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub enum InfoChunkEnum {
    Iarl(IarlChunk),
    Ignr(IgnrChunk),
    Ikey(IkeyChunk),
    Ilgt(IlgtChunk),
    Imed(ImedChunk),
    Inam(InamChunk),
    Iplt(IpltChunk),
    Iprd(IprdChunk),
    Isbj(IsbjChunk),
    Isft(IsftChunk),
    Ishp(IshpChunk),
    Iart(IartChunk),
    Isrc(IsrcChunk),
    Isrf(IsrfChunk),
    Itch(ItchChunk),
    Icms(IcmsChunk),
    Icmt(IcmtChunk),
    Icop(IcopChunk),
    Icrd(IcrdChunk),
    Icrp(IcrpChunk),
    Idpi(IdpiChunk),
    Ieng(IengChunk),
    Unknown {
        id: FourCC,
        size: u32,
        #[brw(align_after=2, pad_size_to= size.to_owned())]
        value: NullString,
    },
}

impl InfoChunkEnum {
    pub fn id(&self) -> FourCC {
        match self {
            InfoChunkEnum::Iarl(e) => e.id(),
            InfoChunkEnum::Ignr(e) => e.id(),
            InfoChunkEnum::Ikey(e) => e.id(),
            InfoChunkEnum::Ilgt(e) => e.id(),
            InfoChunkEnum::Imed(e) => e.id(),
            InfoChunkEnum::Inam(e) => e.id(),
            InfoChunkEnum::Iplt(e) => e.id(),
            InfoChunkEnum::Iprd(e) => e.id(),
            InfoChunkEnum::Isbj(e) => e.id(),
            InfoChunkEnum::Isft(e) => e.id(),
            InfoChunkEnum::Ishp(e) => e.id(),
            InfoChunkEnum::Iart(e) => e.id(),
            InfoChunkEnum::Isrc(e) => e.id(),
            InfoChunkEnum::Isrf(e) => e.id(),
            InfoChunkEnum::Itch(e) => e.id(),
            InfoChunkEnum::Icms(e) => e.id(),
            InfoChunkEnum::Icmt(e) => e.id(),
            InfoChunkEnum::Icop(e) => e.id(),
            InfoChunkEnum::Icrd(e) => e.id(),
            InfoChunkEnum::Icrp(e) => e.id(),
            InfoChunkEnum::Idpi(e) => e.id(),
            InfoChunkEnum::Ieng(e) => e.id(),
            InfoChunkEnum::Unknown { id, .. } => *id,
        }
    }

    pub fn value(&self) -> String {
        match self {
            InfoChunkEnum::Iarl(e) => e.data.value.to_string(),
            InfoChunkEnum::Ignr(e) => e.data.value.to_string(),
            InfoChunkEnum::Ikey(e) => e.data.value.to_string(),
            InfoChunkEnum::Ilgt(e) => e.data.value.to_string(),
            InfoChunkEnum::Imed(e) => e.data.value.to_string(),
            InfoChunkEnum::Inam(e) => e.data.value.to_string(),
            InfoChunkEnum::Iplt(e) => e.data.value.to_string(),
            InfoChunkEnum::Iprd(e) => e.data.value.to_string(),
            InfoChunkEnum::Isbj(e) => e.data.value.to_string(),
            InfoChunkEnum::Isft(e) => e.data.value.to_string(),
            InfoChunkEnum::Ishp(e) => e.data.value.to_string(),
            InfoChunkEnum::Iart(e) => e.data.value.to_string(),
            InfoChunkEnum::Isrc(e) => e.data.value.to_string(),
            InfoChunkEnum::Isrf(e) => e.data.value.to_string(),
            InfoChunkEnum::Itch(e) => e.data.value.to_string(),
            InfoChunkEnum::Icms(e) => e.data.value.to_string(),
            InfoChunkEnum::Icmt(e) => e.data.value.to_string(),
            InfoChunkEnum::Icop(e) => e.data.value.to_string(),
            InfoChunkEnum::Icrd(e) => e.data.value.to_string(),
            InfoChunkEnum::Icrp(e) => e.data.value.to_string(),
            InfoChunkEnum::Idpi(e) => e.data.value.to_string(),
            InfoChunkEnum::Ieng(e) => e.data.value.to_string(),
            InfoChunkEnum::Unknown { value, .. } => format!("Unknown(\"{}\")", *value),
        }
    }
}

#[binrw]
#[br(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct ListAdtlChunkData {
    #[brw(assert(list_type == FourCC(*b"adtl")))]
    list_type: FourCC,
    #[br(parse_with = helpers::until_eof)]
    #[bw()]
    // chunks: Vec<AdtlChunk>,
    raw: Vec<u8>,
}

impl KnownChunkID for ListAdtlChunkData {
    const ID: FourCC = FourCC(*b"LIST");
}

type ListAdtlChunk = KnownChunk<ListAdtlChunkData>;

impl ListAdtlChunk {
    pub fn summary(&self) -> String {
        format!("{} ...", self.data.list_type)
    }
}

// BEXT, based on https://tech.ebu.ch/docs/tech/tech3285.pdf
// BEXT is specified to use ASCII, but we're parsing it as utf8, since
// that is a superset of ASCII and many WAV files contain utf8 strings here
#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub struct BextChunkData {
    /// Description of the sound sequence
    description: FixedStr<256>, // Description
    /// Name of the originator
    originator: FixedStr<32>, // Originator
    /// Reference of the originator
    originator_reference: FixedStr<32>, // OriginatorReference
    /// yyyy:mm:dd
    origination_date: FixedStr<10>, // OriginationDate
    /// hh:mm:ss
    origination_time: FixedStr<8>, // OriginationTime
    /// First sample count since midnight
    time_reference: u64, // TimeReference
    /// Version of the BWF; unsigned binary number
    version: u16, // Version
    /// SMPTE UMID, raw unparsed data
    umid: [u8; 64], // UMID
    /// Integrated Loudness Value of the file in LUFS (multiplied by 100)
    loudness_value: i16, // LoudnessValue
    /// Integrated Loudness Range of the file in LUFS (multiplied by 100)
    loudness_range: i16, // LoudnessRange
    /// Maximum True Peak Level of the file expressed as dBTP (multiplied by 100)
    max_true_peak_level: i16, // MaxTruePeakLevel
    /// Highest value of the Momentary Loudness Level of the file in LUFS (multiplied by 100)
    max_momentary_loudness: i16, // MaxMomentaryLoudness
    /// Highest value of the Short-Term Loudness Level of the file in LUFS (multiplied by 100)
    max_short_term_loudness: i16, // MaxShortTermLoudness
    /// 180 bytes, reserved for future use, set to “NULL”
    reserved: [u8; 180], // Reserved
    /// History coding
    // interpret the remaining bytes as string
    #[br(parse_with = helpers::until_eof, map = |v: Vec<u8>| String::from_utf8_lossy(&v).to_string())]
    #[bw(map = |s: &String| s.as_bytes())]
    coding_history: String, // CodingHistory
}

impl KnownChunkID for BextChunkData {
    const ID: FourCC = FourCC(*b"bext");
}

type BextChunk = KnownChunk<BextChunkData>;

impl BextChunk {
    pub fn summary(&self) -> String {
        format!(
            "{}, {}, {}",
            self.data.origination_date, self.data.origination_time, self.data.description
        )
    }

    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        Box::new(self.into_iter())
    }
}

impl<'a> IntoIterator for &'a BextChunk {
    type Item = (String, String);
    type IntoIter = BextChunkIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BextChunkIterator {
            bext: self,
            index: 0,
        }
    }
}

pub struct BextChunkIterator<'a> {
    bext: &'a BextChunk,
    index: usize,
}

impl<'a> Iterator for BextChunkIterator<'a> {
    type Item = (String, String);
    fn next(&mut self) -> Option<(String, String)> {
        self.index += 1;
        match self.index {
            1 => Some((
                "description".to_string(),
                self.bext.data.description.to_string(),
            )),
            2 => Some((
                "originator".to_string(),
                self.bext.data.originator.to_string(),
            )),
            3 => Some((
                "originator_reference".to_string(),
                self.bext.data.originator_reference.to_string(),
            )),
            4 => Some((
                "origination_date".to_string(),
                self.bext.data.origination_date.to_string(),
            )),
            5 => Some((
                "origination_time".to_string(),
                self.bext.data.origination_time.to_string(),
            )),
            6 => Some((
                "time_reference".to_string(),
                self.bext.data.time_reference.to_string(),
            )),
            7 => Some(("version".to_string(), self.bext.data.version.to_string())),
            8 => Some(("umid".to_string(), hex::encode(self.bext.data.umid))),
            9 => Some((
                "loudness_value".to_string(),
                self.bext.data.loudness_value.to_string(),
            )),
            10 => Some((
                "loudness_range".to_string(),
                self.bext.data.loudness_range.to_string(),
            )),
            11 => Some((
                "max_true_peak_level".to_string(),
                self.bext.data.max_true_peak_level.to_string(),
            )),
            12 => Some((
                "max_momentary_loudness".to_string(),
                self.bext.data.max_momentary_loudness.to_string(),
            )),
            13 => Some((
                "max_short_term_loudness".to_string(),
                self.bext.data.max_short_term_loudness.to_string(),
            )),
            14 => Some((
                "coding_history".to_string(),
                self.bext.data.coding_history.to_string(),
            )),
            _ => None,
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
pub enum ChunkEnum {
    Fmt(FmtChunk),
    Data(DataChunk),
    Info(ListInfoChunk),
    Adtl(ListAdtlChunk),
    Bext(Box<BextChunk>),
    Md5(Md5Chunk),
    Unknown {
        id: FourCC,
        size: u32,
        #[br(count = size )]
        #[bw()]
        raw: Vec<u8>,
    },
}

impl ChunkID for ChunkEnum {
    /// Returns the [FourCC] (chunk id) for the contained chunk.
    fn id(&self) -> FourCC {
        match self {
            ChunkEnum::Fmt(e) => e.id(),
            ChunkEnum::Data(e) => e.id(),
            ChunkEnum::Info(e) => e.id(),
            ChunkEnum::Adtl(e) => e.id(),
            ChunkEnum::Bext(e) => e.id(),
            ChunkEnum::Md5(e) => e.id(),
            ChunkEnum::Unknown { id, .. } => *id,
        }
    }
}

impl SizedChunk for ChunkEnum {
    /// Returns the logical (used) size in bytes of the contained chunk.
    fn size(&self) -> u32 {
        match self {
            ChunkEnum::Fmt(e) => e.size,
            ChunkEnum::Data(e) => e.size,
            ChunkEnum::Info(e) => e.size,
            ChunkEnum::Adtl(e) => e.size,
            ChunkEnum::Bext(e) => e.size,
            ChunkEnum::Md5(e) => e.size,
            ChunkEnum::Unknown { size, .. } => *size,
        }
    }
}

impl Summarizable for ChunkEnum {
    /// Returns a short text summary of the contents of the contained chunk.
    fn summary(&self) -> String {
        match self {
            ChunkEnum::Fmt(e) => e.summary(),
            ChunkEnum::Data(e) => e.summary(),
            ChunkEnum::Info(e) => e.summary(),
            ChunkEnum::Adtl(e) => e.summary(),
            ChunkEnum::Bext(e) => e.summary(),
            ChunkEnum::Md5(e) => e.summary(),
            ChunkEnum::Unknown { .. } => "...".to_owned(),
        }
    }

    /// Returns an iterator over a sequence of contents of the contained
    /// chunk as strings (field, value).
    fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        match self {
            ChunkEnum::Fmt(e) => Box::new(e.into_iter()),
            ChunkEnum::Info(e) => Box::new(e.items()),
            ChunkEnum::Bext(e) => Box::new(e.items()),
            _ => Box::new(std::iter::empty()),
        }
    }
}

impl Chunk for ChunkEnum {}

#[cfg(test)]
mod test {
    use binrw::BinRead; // don't understand why this is needed in this scope
    use std::io::Cursor;

    use super::*;
    use hex::decode;
    use hexdump::hexdump;

    fn hex_to_cursor(data: &str) -> Cursor<Vec<u8>> {
        let data = data.replace(' ', "");
        let data = data.replace('\n', "");
        let data = decode(data).unwrap();
        Cursor::new(data)
    }

    #[test]
    fn fixed_string() {
        let fs = FixedStr::<6>(*b"abc\0\0\0");
        assert_eq!(6, fs.0.len());
        let s = fs.to_string();
        assert_eq!("abc".to_string(), s);
        assert_eq!(3, s.len());
        let new_fs = FixedStr::<6>::from_str(&s).unwrap();
        assert_eq!(fs, new_fs);
        assert_eq!(6, fs.0.len());
        assert_eq!(
            b"\0\0\0"[..],
            new_fs.0[3..6],
            "extra space after the string should be null bytes"
        );

        // strings longer than fixed size should get truncated
        let long_str = "this is a longer str";
        let fs = FixedStr::<6>::from_str(long_str).unwrap();
        assert_eq!(fs.to_string(), "this i");
    }

    #[test]
    fn riff_header() {
        // RIFF 2398 WAVE
        let header = "524946465E09000057415645";
        let mut data = hex_to_cursor(header);
        println!("{header:?}");
        let wavfile = RiffChunk::read(&mut data).unwrap();
        assert_eq!(
            wavfile,
            RiffChunk {
                id: FourCC(*b"RIFF"),
                size: 2398,
                form_type: FourCC(*b"WAVE"),
            }
        );
    }

    #[test]
    fn parse_fmt() {
        let mut buff = hex_to_cursor("666D7420 10000000 01000100 80BB0000 80320200 03001800");
        let expected = FmtChunk {
            size: 16,
            data: FmtChunkData {
                format_tag: FormatTag::Pcm,
                channels: 1,
                samples_per_sec: 48000,
                avg_bytes_per_sec: 144000,
                block_align: 3,
                bits_per_sample: 24,
            },
            extra_bytes: vec![],
        };
        let chunk = FmtChunk::read(&mut buff).expect("error parsing WAV chunks");
        assert_eq!(chunk, expected);
        // hexdump(remaining_input);
    }

    #[test]
    fn parse_bext() {
        // example bext chunk data from BWF MetaEdit
        let mut buff = hex_to_cursor(
            r#"62657874 67020000 44657363 72697074 696F6E00 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 4F726967 696E6174 6F720000 
            00000000 00000000 00000000 00000000 00000000 4F726967 696E6174 
            6F725265 66657265 6E636500 00000000 00000000 00000000 32303036 
            2F30312F 30323033 3A30343A 30353930 00000000 00000200 060A2B34 
            01010101 01010210 13000000 00FF122A 69370580 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 6400C800 2C019001 F4010000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 00000000 
            00000000 00000000 00000000 00000000 00000000 00000000 0000436F 
            64696E67 48697374 6F7279"#,
        );
        let bext = BextChunk::read(&mut buff).expect("error parsing bext chunk");
        print!("{:?}", bext);
        assert_eq!(
            bext.data.description,
            FixedStr::<256>::from_str("Description").unwrap(),
            "description"
        );
        assert_eq!(
            bext.data.originator,
            FixedStr::<32>::from_str("Originator").unwrap(),
            "originator"
        );
        assert_eq!(
            bext.data.originator_reference,
            FixedStr::<32>::from_str("OriginatorReference").unwrap(),
            "originator_reference"
        );
        assert_eq!(
            bext.data.origination_date,
            FixedStr::<10>::from_str("2006/01/02").unwrap(),
            "origination_date"
        );
        assert_eq!(
            bext.data.origination_time,
            FixedStr::<8>::from_str("03:04:05").unwrap(),
            "origination_time"
        );
        assert_eq!(bext.data.time_reference, 12345, "time_reference");
        assert_eq!(bext.data.version, 2);
        assert_eq!(
            bext.data.umid,
            <Vec<u8> as TryInto<[u8; 64]>>::try_into(
                decode("060A2B3401010101010102101300000000FF122A6937058000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()
            )
            .unwrap(),
            "version"
        );
        assert_eq!(bext.data.loudness_value, 100, "loudness_value");
        assert_eq!(bext.data.loudness_range, 200, "loudness_range");
        assert_eq!(bext.data.max_true_peak_level, 300, "max_true_peak_level");
        assert_eq!(
            bext.data.max_momentary_loudness, 400,
            "max_momentary_loudness"
        );
        assert_eq!(
            bext.data.max_short_term_loudness, 500,
            "max_short_term_loudness"
        );
        assert_eq!(bext.data.reserved.len(), 180, "reserved");
        assert_eq!(bext.data.coding_history, "CodingHistory", "coding_history");
    }

    #[test]
    fn decode_spaces() {
        let a = &decode("666D7420 10000000 01000100 80BB0000 80320200 03001800".replace(' ', ""))
            .unwrap();
        let b = &decode("666D7420100000000100010080BB00008032020003001800").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn parse_fixedstr_data_after_zero() {
        // REAPER had (still has?) a bug where data from other BEXT fields
        // can be left in a fixed lenth string field after the terminating
        // zero byte. This input is an example that starts with "REAPER"
        // but has part of a path string after the terminating zero.
        let mut buff = hex_to_cursor(
            "52454150 45520065 72732F62 7269616E 2F70726F 6A656374 732F7761 7672772F",
        );
        let fs = FixedStr::<32>::read_options(&mut buff, binrw::Endian::Big, ())
            .expect("error parsing FixedStr");
        assert_eq!(fs, FixedStr::<32>::from_str("REAPER").unwrap());
    }

    #[test]
    fn infochunk_roundtrip() {
        let icmt = InfoChunkEnum::Icmt(IcmtChunk {
            size: 8,
            data: IcmtChunkData {
                value: NullString("comment".into()),
            },
            extra_bytes: vec![],
        });
        println!("{icmt:?}");
        let mut buff = std::io::Cursor::new(Vec::<u8>::new());
        icmt.write(&mut buff).unwrap();
        println!("{:?}", hexdump(buff.get_ref()));
        buff.set_position(0);
        let after = InfoChunkEnum::read(&mut buff).unwrap();
        assert_eq!(after, icmt);
    }

    #[test]
    fn infochunk_small_valid() {
        // buff contains ICMT chunk with an odd length
        // handling the WORD padding incorrectly can break parsing
        let mut buff =
            hex_to_cursor("49434D54 15000000 62657874 20636875 6E6B2074 65737420 66696C65 00");
        // parse via explicit chunk type
        let icmt = IcmtChunk::read(&mut buff).unwrap();
        dbg!(&icmt);
        assert_eq!(icmt.id(), FourCC(*b"ICMT"));
        assert_eq!(icmt.data.value, "bext chunk test file".into());

        // parse via enum wrapper this time
        buff.set_position(0);
        let en = InfoChunkEnum::read(&mut buff).unwrap();
        dbg!(&en);
        assert_eq!(en.id(), FourCC(*b"ICMT"));
        let InfoChunkEnum::Icmt(icmt) = en else {
            unreachable!("should have been ICMT")
        };
        assert_eq!(icmt.data.value, "bext chunk test file".into());
    }

    #[test]
    fn listinfochunk_small_valid() {
        // buff contains INFO chunk with two odd length'd inner chunks
        // handling the WORD padding incorrectly can break parsing
        // if infochunk_small_valid() passes, but this fails, error is
        // likely in the ListInfoChunk wrapping
        let mut buff = hex_to_cursor(
        "4C495354 38000000 494E464F 49534654 0D000000 42574620 4D657461 45646974 00004943 4D541500 00006265 78742063 68756E6B 20746573 74206669 6C6500"
            );
        let list = ListInfoChunk::read(&mut buff).unwrap();

        // parse via enum wrapper this time
        buff.set_position(0);
        let chunk = ChunkEnum::read(&mut buff).map(box_chunk).unwrap();
        assert_eq!(chunk.id(), FourCC(*b"LIST"));

        // let list = ListInfoChunk::read(&mut buff).unwrap();
        // assert_eq!(
    }

    #[test]
    fn infochunk_debug_string() {
        let icmt = IcmtChunkData {
            value: NullString("comment".into()),
        };
        assert!(format!("{icmt:?}").starts_with("InfoChunkData<ICMT>"));
    }

    #[test]
    fn icmtchunk_as_trait() {
        let icmt = IcmtChunk {
            size: 8,
            data: IcmtChunkData::new("comment".into()),
            extra_bytes: vec![],
        };
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn Chunk> = Box::new(icmt);
    }

    #[test]
    fn knownchunk_as_trait() {
        let md5 = Md5Chunk {
            size: 16,
            data: Md5ChunkData { md5: 0 },
            extra_bytes: vec![],
        };
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn Chunk> = Box::new(md5);
    }

    #[test]
    fn chunkenum_as_trait() {
        let md5 = ChunkEnum::Md5(Md5Chunk {
            size: 16,
            data: Md5ChunkData { md5: 0 },
            extra_bytes: vec![],
        });
        // ensure trait bounds are satisfied
        let mut _trt: Box<dyn Chunk> = Box::new(md5);
    }
}
