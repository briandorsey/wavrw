// use core::fmt;
use alloc::collections::BTreeMap;
use core::fmt::Debug;

/// `ASWG` Audio Standards Working Group (ASWG) metadata.
/// [ASWG-G006](https://wavref.til.cafe/spec/aswg-g006/)
///
/// Represents iXML `ASWG` values as defined in [ASWG-G006 - iXML Extension Specification v1.1.pdf](https://github.com/Sony-ASWG/iXML-Extension/blob/main/ASWG-G006%20-%20iXML%20Extension%20Specification%20v1.1.pdf)
///
/// Version 1.1 is a superset of 1.0 and all fields are optional, so we're defining
/// all the 1.1 fields.
///
/// Description from standards doc:
/// > The ASWG iXML Extension is designed to provide developers of interactive
/// > audio content and audio researchers the ability to store production and
/// > research related metadata within the `<BWFXML>` chunk of a Broadcast Wave
/// > file, describing its contents and other related information.
/// >
/// > The specification defines fields relating to metadata that can be used
/// > within interactive media development applications and workflows as well
/// > as machine learning and deep learning feature sets.
/// >
/// > The extension contains fields covering sound effects, music, dialogue
/// > and audio-driven haptic content, as well as more general, project
/// > information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Aswg {
    /// Content Type (sfx/music/dialog/haptic/impulse/mixed). category:General
    pub content_type: Option<String>,

    /// Project name asset was developed for. category:General
    pub project: Option<String>,

    /// Designer. category:General
    pub originator: Option<String>,

    /// Name of originating studio. category:General
    pub originator_studio: Option<String>,

    /// General information not covered in other fields. category:General
    pub notes: Option<String>,

    /// Application (Pro Tools/Reaper etc.) session name. category:General
    pub session: Option<String>,

    /// File version: mastered, processed, raw, placeholder. category:General
    pub state: Option<String>,

    /// Name of editor. category:General
    pub editor: Option<String>,

    /// Mix engineer. category:General
    pub mixer: Option<String>,

    /// Name of FX chain used on file, Reaper chain name, for example. category:General
    pub fx_chain_name: Option<String>,

    /// Content is AI generated, or contains elements/sections that are AI generated. true/false. category:General
    pub is_generated: Option<String>,

    /// Name of the mastering engineer. category:General
    pub mastering_engineer: Option<String>,

    /// Date of original upload of asset in format yyyy-MM-dd. category:General
    pub origination_date: Option<String>,

    /// Channel configuration of the file: mono, stereo, LCR, Quad, 5.0, 5.1, 7.0, 7.1, 12.2, ambisonic. category:Format
    pub channel_config: Option<String>,

    /// Ambisonic format: #p, #h#p, #h#v.  eg: 5p, 3h1v, 4h2p. category:Format" format="#p, #h#p, #h#v.  eg: '5p', '3h1v', '4h2p'
    pub ambisonic_format: Option<String>,

    /// Ambisonic channel order: fuma, acn. category:Format
    pub ambisonic_chn_order: Option<String>,

    /// Ambisonic normalization: snd3, maxn, n3d. category:Format
    pub ambisonic_norm: Option<String>,

    /// Microphone(s) used.  Where multiple mics used, prefix with channel number: 1-Neumann U87i, 2-AKG C414. category:Recording
    pub mic_type: Option<String>,

    /// Microphone configuration: Mono, AB, XY, ORTF, MS. category:Recording
    pub mic_config: Option<String>,

    /// Microphone distance in meters OR headmounted - 1m, 2m, 0.3m, head. category:Recording
    pub mic_distance: Option<String>,

    /// Recording location. category:Recording
    pub recording_loc: Option<String>,

    /// SFX: Is the sound designed, or is it a raw recording - true if designed, false if raw recording. category:Recording
    pub is_designed: Option<String>,

    /// Name of the recording engineer. category:Recording
    pub rec_engineer: Option<String>,

    /// Music: Recording Studio. category:Recording
    pub rec_studio: Option<String>,

    /// Impulse: Location of impulse. category:Impulse
    pub impulse_location: Option<String>,

    /// UCS compliant SFX category. category:Sound Effects
    pub category: Option<String>,

    /// UCS compliant SFX sub-category. category:Sound Effects
    pub sub_category: Option<String>,

    /// UCS compliant SFX category ID. category:Sound Effects
    pub cat_id: Option<String>,

    /// UCS complaint user category. category:Sound Effects
    pub user_category: Option<String>,

    /// UCS compliant user data. category:Sound Effects
    pub user_data: Option<String>,

    /// UCS compliant vendor category. category:Sound Effects
    pub vendor_category: Option<String>,

    /// UCS compliant FX name. category:Sound Effects
    pub fx_name: Option<String>,

    /// UCS compliant library. category:Sound Effects
    pub library: Option<String>,

    /// UCS compliant SFX creator/publisher. category:Sound Effects
    pub creator_id: Option<String>,

    /// UCS compliant SFX `SourceID`. category:Sound Effects
    pub source_id: Option<String>,

    /// RMS power of file. category:Audio Features
    pub rms_power: Option<String>,

    /// Integrated loudness of file, measured with ITU-R BS1770-3 compliant metering. category:Audio Features
    pub loudness: Option<String>,

    /// Loudness Range - EBU 3342 compliant. category:Audio Features
    pub loudness_range: Option<String>,

    /// Maximum sample value, in dBFS. category:Audio Features
    pub max_peak: Option<String>,

    /// Spectral density of file - amount of power at a standard set of frequency ranges.  Freq ranges to be defined***. category:Audio Features
    pub spec_density: Option<String>,

    /// Zero Cross Rate, average frequency of entire file. category:Audio Features
    pub zero_cross_rate: Option<String>,

    /// Peak to average power ratio. category:Audio Features
    pub papr: Option<String>,

    /// Dialogue: Transcript of the dialogue file. category:Dialogue
    pub text: Option<String>,

    /// Dialogue: Whether the file contains efforts, dialogue or a mix of the two - True, False, Mixed. category:Dialogue
    pub efforts: Option<String>,

    /// Effort type - strain, pain. category:Dialogue
    pub effort_type: Option<String>,

    /// Dialogue projection level.  1- whispered, 2- spoken, 3- raised, 4- projected, 5- shouted. category:Dialogue
    pub projection: Option<String>,

    /// Dialogue language - ISO639-1 Language Code. category:Dialogue" format="## e.g 'en'
    pub language: Option<String>,

    /// Dialogue timing restriction: wild, time, lip, na (not applicable). category:Dialogue
    pub timing_restriction: Option<String>,

    /// Dialogue: Character name for dialogue files. category:Dialogue
    pub character_name: Option<String>,

    /// Dialogue: Sex/gender of character. category:Dialogue
    pub character_gender: Option<String>,

    /// Dialogue: Age of (human) character. category:Dialogue
    pub character_age: Option<String>,

    /// Dialogue: Whether the character is a main (significant) character or a background character: significant, background. category:Dialogue
    pub character_role: Option<String>,

    /// Dialogue: Name of actor. category:Dialogue
    pub actor_name: Option<String>,

    /// Dialogue: Sex/gender of actor: male, female. category:Dialogue
    pub actor_gender: Option<String>,

    /// Dialogue: Name of director. category:Dialogue
    pub director: Option<String>,

    /// Director’s notes, for context; explaining the scene and character motivation.. category:Dialogue
    pub direction: Option<String>,

    /// Effects used on file eg. Radio. category:Dialogue
    pub fx_used: Option<String>,

    /// Dialogue: Code for usage rights of content: *Internal. category:Dialogue
    pub usage_rights: Option<String>,

    /// Dialogue: Was recording done under a union contract: true, false. category:Dialogue
    pub is_union: Option<String>,

    /// Regional accent of the spoken dialogue, if applicable. category:Dialogue
    pub accent: Option<String>,

    /// Emotional content present in the delivery of the dialogue. category:Dialogue
    pub emotion: Option<String>,

    /// Gender of addressee; male/female/malegroup/femalegroup/mixedgroup. category:Dialogue
    pub addressee_gender: Option<String>,

    /// Either formal or informal, depending on the relationship between the speaker and the addressee. formal/informal. category:Dialogue
    pub is_formal: Option<String>,

    /// Original language used by developer. category:Dialogue
    pub dev_language: Option<String>,

    /// Music: project billing code. category:Music
    pub billing_code: Option<String>,

    /// Music: Composer. category:Music
    pub composer: Option<String>,

    /// Music: Name of artist . category:Music
    pub artist: Option<String>,

    /// Music: Song title. category:Music
    pub song_title: Option<String>,

    /// Music: Genre. category:Music
    pub genre: Option<String>,

    /// Music: Sub-genre. category:Music
    pub sub_genre: Option<String>,

    /// Music: Producer name. category:Music
    pub producer: Option<String>,

    /// Music: Music supervisor. category:Music
    pub music_sup: Option<String>,

    /// Music: Instrument on track/stem. category:Music
    pub instrument: Option<String>,

    /// Music: Name of publisher. category:Music
    pub music_publisher: Option<String>,

    /// Music: Owner of the recorded work. category:Music
    pub rights_owner: Option<String>,

    /// Music: Is this an asset as the composer delivered (source) or an edit of that source?  true, false. category:Music
    pub is_source: Option<String>,

    /// Is the content loopable - true, false. category:Music
    pub is_loop: Option<String>,

    /// Music: intensity. category:Music
    pub intensity: Option<String>,

    /// Music: Is cue temp or final. category:Music
    pub is_final: Option<String>,

    /// Order reference of cue, if applicable *Internal. category:Music
    pub order_ref: Option<String>,

    /// Music: Is part of the Original Soundtrack. category:Music
    pub is_ost: Option<String>,

    /// Music: Asset is associated with a cinematic. category:Music
    pub is_cinematic: Option<String>,

    /// Music: Asset is licensed and owned by 3rd party. category:Music
    pub is_licensed: Option<String>,

    /// Music: Track is diegetic in game. category:Music
    pub is_diegetic: Option<String>,

    /// Music: Version number. category:Music
    pub music_version: Option<String>,

    /// Music: ISRC code. category:Music" format="## ### ## ##### e.g 'UK AAA 05 00001'
    pub isrc_id: Option<String>,

    /// Music: Tempo in bpm. category:Music
    pub tempo: Option<String>,

    /// Music: Time Signature. e.g 3:4. category:Music" format="A:B e.g '3:4'
    pub time_sig: Option<String>,

    /// Music: In key. category:Music
    pub in_key: Option<String>,

    /// Additional tags found in the XML document beyond those listed in
    /// the ASWG spec.
    pub extra: BTreeMap<String, String>,
}

impl Aswg {
    /// User friendly name of the element's key
    pub fn name(&self) -> String {
        "ASWG".to_string()
    }

    /// Create a new Aswg struct.
    pub fn new() -> Self {
        Aswg {
            content_type: None,
            project: None,
            originator: None,
            originator_studio: None,
            notes: None,
            session: None,
            state: None,
            editor: None,
            mixer: None,
            fx_chain_name: None,
            is_generated: None,
            mastering_engineer: None,
            origination_date: None,
            channel_config: None,
            ambisonic_format: None,
            ambisonic_chn_order: None,
            ambisonic_norm: None,
            mic_type: None,
            mic_config: None,
            mic_distance: None,
            recording_loc: None,
            is_designed: None,
            rec_engineer: None,
            rec_studio: None,
            impulse_location: None,
            category: None,
            sub_category: None,
            cat_id: None,
            user_category: None,
            user_data: None,
            vendor_category: None,
            fx_name: None,
            library: None,
            creator_id: None,
            source_id: None,
            rms_power: None,
            loudness: None,
            loudness_range: None,
            max_peak: None,
            spec_density: None,
            zero_cross_rate: None,
            papr: None,
            text: None,
            efforts: None,
            effort_type: None,
            projection: None,
            language: None,
            timing_restriction: None,
            character_name: None,
            character_gender: None,
            character_age: None,
            character_role: None,
            actor_name: None,
            actor_gender: None,
            director: None,
            direction: None,
            fx_used: None,
            usage_rights: None,
            is_union: None,
            accent: None,
            emotion: None,
            addressee_gender: None,
            is_formal: None,
            dev_language: None,
            billing_code: None,
            composer: None,
            artist: None,
            song_title: None,
            genre: None,
            sub_genre: None,
            producer: None,
            music_sup: None,
            instrument: None,
            music_publisher: None,
            rights_owner: None,
            is_source: None,
            is_loop: None,
            intensity: None,
            is_final: None,
            order_ref: None,
            is_ost: None,
            is_cinematic: None,
            is_licensed: None,
            is_diegetic: None,
            music_version: None,
            isrc_id: None,
            tempo: None,
            time_sig: None,
            in_key: None,
            extra: BTreeMap::new(),
        }
    }

    /// Helper to set value by path.
    pub fn set(&mut self, path: &[String], value: String) {
        if let Some(first) = path.first() {
            match first.as_str() {
                "contentType" => self.content_type = Some(value),
                "project" => self.project = Some(value),
                "originator" => self.originator = Some(value),
                "originatorStudio" => self.originator_studio = Some(value),
                "notes" => self.notes = Some(value),
                "session" => self.session = Some(value),
                "state" => self.state = Some(value),
                "editor" => self.editor = Some(value),
                "mixer" => self.mixer = Some(value),
                "fxChainName" => self.fx_chain_name = Some(value),
                "isGenerated" => self.is_generated = Some(value),
                "masteringEngineer" => self.mastering_engineer = Some(value),
                "originationDate" => self.origination_date = Some(value),
                "channelConfig" => self.channel_config = Some(value),
                "ambisonicFormat" => self.ambisonic_format = Some(value),
                "ambisonicChnOrder" => self.ambisonic_chn_order = Some(value),
                "ambisonicNorm" => self.ambisonic_norm = Some(value),
                "micType" => self.mic_type = Some(value),
                "micConfig" => self.mic_config = Some(value),
                "micDistance" => self.mic_distance = Some(value),
                "recordingLoc" => self.recording_loc = Some(value),
                "isDesigned" => self.is_designed = Some(value),
                "recEngineer" => self.rec_engineer = Some(value),
                "recStudio" => self.rec_studio = Some(value),
                "impulseLocation" => self.impulse_location = Some(value),
                "category" => self.category = Some(value),
                "subCategory" => self.sub_category = Some(value),
                "catId" => self.cat_id = Some(value),
                "userCategory" => self.user_category = Some(value),
                "userData" => self.user_data = Some(value),
                "vendorCategory" => self.vendor_category = Some(value),
                "fxName" => self.fx_name = Some(value),
                "library" => self.library = Some(value),
                "creatorId" => self.creator_id = Some(value),
                "sourceId" => self.source_id = Some(value),
                "rmsPower" => self.rms_power = Some(value),
                "loudness" => self.loudness = Some(value),
                "loudnessRange" => self.loudness_range = Some(value),
                "maxPeak" => self.max_peak = Some(value),
                "specDensity" => self.spec_density = Some(value),
                "zeroCrossRate" => self.zero_cross_rate = Some(value),
                "papr" => self.papr = Some(value),
                "text" => self.text = Some(value),
                "efforts" => self.efforts = Some(value),
                "effortType" => self.effort_type = Some(value),
                "projection" => self.projection = Some(value),
                "language" => self.language = Some(value),
                "timingRestriction" => self.timing_restriction = Some(value),
                "characterName" => self.character_name = Some(value),
                "characterGender" => self.character_gender = Some(value),
                "characterAge" => self.character_age = Some(value),
                "characterRole" => self.character_role = Some(value),
                "actorName" => self.actor_name = Some(value),
                "actorGender" => self.actor_gender = Some(value),
                "director" => self.director = Some(value),
                "direction" => self.direction = Some(value),
                "fxUsed" => self.fx_used = Some(value),
                "usageRights" => self.usage_rights = Some(value),
                "isUnion" => self.is_union = Some(value),
                "accent" => self.accent = Some(value),
                "emotion" => self.emotion = Some(value),
                "addresseeGender" => self.addressee_gender = Some(value),
                "isFormal" => self.is_formal = Some(value),
                "devLanguage" => self.dev_language = Some(value),
                "billingCode" => self.billing_code = Some(value),
                "composer" => self.composer = Some(value),
                "artist" => self.artist = Some(value),
                "songTitle" => self.song_title = Some(value),
                "genre" => self.genre = Some(value),
                "subGenre" => self.sub_genre = Some(value),
                "producer" => self.producer = Some(value),
                "musicSup" => self.music_sup = Some(value),
                "instrument" => self.instrument = Some(value),
                "musicPublisher" => self.music_publisher = Some(value),
                "rightsOwner" => self.rights_owner = Some(value),
                "isSource" => self.is_source = Some(value),
                "isLoop" => self.is_loop = Some(value),
                "intensity" => self.intensity = Some(value),
                "isFinal" => self.is_final = Some(value),
                "orderRef" => self.order_ref = Some(value),
                "isOst" => self.is_ost = Some(value),
                "isCinematic" => self.is_cinematic = Some(value),
                "isLicensed" => self.is_licensed = Some(value),
                "isDiegetic" => self.is_diegetic = Some(value),
                "musicVersion" => self.music_version = Some(value),
                "isrcId" => self.isrc_id = Some(value),
                "tempo" => self.tempo = Some(value),
                "timeSig" => self.time_sig = Some(value),
                "inKey" => self.in_key = Some(value),
                &_ => {
                    self.extra.insert(path.join("/"), value);
                }
            }
        }
    }

    /// Iterate over fields and values as (String, String)
    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = (String, String)> + 'a> {
        let mut items: Vec<(String, String)> = Vec::new();
        let mut push = |key: &str, value: &Option<String>| {
            if let Some(val) = value {
                items.push((key.to_string(), val.clone()));
            }
        };

        push("contentType", &self.content_type);
        push("project", &self.project);
        push("originator", &self.originator);
        push("originatorStudio", &self.originator_studio);
        push("notes", &self.notes);
        push("session", &self.session);
        push("state", &self.state);
        push("editor", &self.editor);
        push("mixer", &self.mixer);
        push("fxChainName", &self.fx_chain_name);
        push("isGenerated", &self.is_generated);
        push("masteringEngineer", &self.mastering_engineer);
        push("originationDate", &self.origination_date);
        push("channelConfig", &self.channel_config);
        push("ambisonicFormat", &self.ambisonic_format);
        push("ambisonicChnOrder", &self.ambisonic_chn_order);
        push("ambisonicNorm", &self.ambisonic_norm);
        push("micType", &self.mic_type);
        push("micConfig", &self.mic_config);
        push("micDistance", &self.mic_distance);
        push("recordingLoc", &self.recording_loc);
        push("isDesigned", &self.is_designed);
        push("recEngineer", &self.rec_engineer);
        push("recStudio", &self.rec_studio);
        push("impulseLocation", &self.impulse_location);
        push("category", &self.category);
        push("subCategory", &self.sub_category);
        push("catId", &self.cat_id);
        push("userCategory", &self.user_category);
        push("userData", &self.user_data);
        push("vendorCategory", &self.vendor_category);
        push("fxName", &self.fx_name);
        push("library", &self.library);
        push("creatorId", &self.creator_id);
        push("sourceId", &self.source_id);
        push("rmsPower", &self.rms_power);
        push("loudness", &self.loudness);
        push("loudnessRange", &self.loudness_range);
        push("maxPeak", &self.max_peak);
        push("specDensity", &self.spec_density);
        push("zeroCrossRate", &self.zero_cross_rate);
        push("papr", &self.papr);
        push("text", &self.text);
        push("efforts", &self.efforts);
        push("effortType", &self.effort_type);
        push("projection", &self.projection);
        push("language", &self.language);
        push("timingRestriction", &self.timing_restriction);
        push("characterName", &self.character_name);
        push("characterGender", &self.character_gender);
        push("characterAge", &self.character_age);
        push("characterRole", &self.character_role);
        push("actorName", &self.actor_name);
        push("actorGender", &self.actor_gender);
        push("director", &self.director);
        push("direction", &self.direction);
        push("fxUsed", &self.fx_used);
        push("usageRights", &self.usage_rights);
        push("isUnion", &self.is_union);
        push("accent", &self.accent);
        push("emotion", &self.emotion);
        push("addresseeGender", &self.addressee_gender);
        push("isFormal", &self.is_formal);
        push("devLanguage", &self.dev_language);
        push("billingCode", &self.billing_code);
        push("composer", &self.composer);
        push("artist", &self.artist);
        push("songTitle", &self.song_title);
        push("genre", &self.genre);
        push("subGenre", &self.sub_genre);
        push("producer", &self.producer);
        push("musicSup", &self.music_sup);
        push("instrument", &self.instrument);
        push("musicPublisher", &self.music_publisher);
        push("rightsOwner", &self.rights_owner);
        push("isSource", &self.is_source);
        push("isLoop", &self.is_loop);
        push("intensity", &self.intensity);
        push("isFinal", &self.is_final);
        push("orderRef", &self.order_ref);
        push("isOst", &self.is_ost);
        push("isCinematic", &self.is_cinematic);
        push("isLicensed", &self.is_licensed);
        push("isDiegetic", &self.is_diegetic);
        push("musicVersion", &self.music_version);
        push("isrcId", &self.isrc_id);
        push("tempo", &self.tempo);
        push("timeSig", &self.time_sig);
        push("inKey", &self.in_key);

        for (k, v) in &self.extra {
            items.push((format!("{} (extra)", k), v.clone()));
        }

        Box::new(items.into_iter())
    }
}

impl Default for Aswg {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::dbg_macro)]
#[cfg(test)]
mod test {

    use super::*;

    #[test]
    // prove to myself that items iteration is working
    fn iterator_for_loop() {
        let mut items: Vec<(String, String)> = vec![];
        let mut aswg = Aswg::new();
        aswg.project = Some("Test Project".to_string());
        for item in aswg.items() {
            items.push(item);
        }
        assert_eq!(
            vec![("project".to_string(), "Test Project".to_string())],
            items
        );
    }

    #[test]
    fn items() {
        let mut aswg = Aswg::new();
        aswg.project = Some("Test Project".to_string());
        let result: Vec<(String, String)> = aswg.items().collect();
        print!("{:?}", result);
        assert!(result.contains(&("project".to_string(), "Test Project".to_string())));
    }
}
