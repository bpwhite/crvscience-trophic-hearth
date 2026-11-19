// Copyright (C) 2026 CRV Science Network (https://crvscience.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://gnu.org>.

use crate::trophic::{BiomeKind, TrophicStratum};
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Canonical species across Neotropical Rainforest, Pacific Giant Kelp Forest, and Indo-Pacific Coral Reef.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Species {
    // === NEOTROPICAL RAINFOREST ===
    AmanitaFungus,
    MycenaBiolumFungus,
    LeafcutterAnt,
    ThreeToedSloth,
    MorphoButterfly,
    CentralAmericanAgouti,
    LowlandTapir,
    SquirrelMonkey,
    ScarletMacaw,
    KeelBilledToucan,
    BoaConstrictor,
    BlackPanther,
    CanopyJaguar,

    // === NORTHEAST PACIFIC GIANT KELP FOREST ===
    CaliforniaSeaCucumber,
    BatSeaStar,
    NorthernKelpCrab,
    PurpleSeaUrchin,
    RedAbalone,
    BlacksmithDamselfish,
    SeaOtter,
    CaliforniaSheephead,
    GaribaldiDamselfish,
    GiantPacificOctopus,
    PacificHarborSeal,
    LeopardShark,
    PacificWolfEel,

    // === INDO-PACIFIC CORAL REEF ===
    // L0: Cleaners & Benthic Detritivores
    PacificCleanerShrimp,
    BlueLinckiaStar,
    ReefHermitCrab,
    // L2: Grazers, Filter Feeders & Zooplanktivores
    GreenSeaTurtle,
    SteepheadParrotfish,
    BlueGreenChromis,
    GiantTridacnaClam,
    // L3: Anemone Mutualists & Reef Mesopredators
    OcellarisClownfish,
    MoorishIdol,
    EmperorAngelfish,
    BluespottedStingray,
    // L4: Apex & Crevice Ambush Predators
    BlacktipReefShark,
    GiantMorayEel,
    RedLionfish,

    // === TEMPERATE FRESHWATER LAKE (SPLIT-LEVEL UNDERWATER + WATERBIRDS) ===
    // L0: Benthic Detritivores
    SignalCrayfish,
    // L2: Herbivores, Minnow Schools & Aerial Odonates
    PumpkinseedSunfish,
    GoldenShiner,
    BlueDasherDragonfly,
    // L3: Dabbling Waterfowl, Perching Piscivores, Salmonids, Centrarchids & Herpetofauna
    MallardDuck,
    BeltedKingfisher,
    RainbowTrout,
    LargemouthBass,
    WesternPaintedTurtle,
    AmericanBullfrog,
    // L4: Aerial Plunge-Divers, Pursuit Diving Waterbirds, Waders & Ambush Esox
    Osprey,
    CommonLoon,
    GreatBlueHeron,
    NorthernPike,

    // === DEEP-SEA HYDROTHERMAL VENT (2,600M BLACK SMOKER) ===
    // L0: Sulfide-Floc Squat Lobsters & Chimney Polychaetes
    MunidopsisSquatLobster,
    // L2: Chemosynthetic Bacterivores (Yeti Crabs, Vent Shrimp & Iron-Plated Snails)
    KiwaYetiCrab,
    RimicarisVentShrimp,
    ScalyFootSnail,
    // L3: Vent Mesopredators & Bioluminescent Midwater Gelatinous Fauna
    ZoarcidVentEelpout,
    AtollaCrownJelly,
    BiolumCombJelly,
    // L4: Bathypelagic & Abyssal Vent Apex Predators
    AbyssalGhostShark,
    BluntnoseSixgillShark,
    VulcanoctopusVent,

    // === PNW OLD-GROWTH & SPAWNING SALMON CREEK ===
    // L0: Nurse-Log Pulmonates, Ectomycorrhizal Fruiting Bodies & Riffle Shredders
    PacificBananaSlug,
    GoldenChanterelle,
    CasedCaddisflyLarva,
    // L2: Anadromous Spawners & Conifer Cone Foragers
    SpawningCohoSalmon,
    DouglasSquirrel,
    // L3: Underwater-Walking Songbird, Stream Giant Amphibian & Understory Thrush
    AmericanDipper,
    PacificGiantSalamander,
    VariedThrush,
    // L4: Old-Growth Canopy Raptor & Riparian Salmon-Vector Carnivore
    NorthernSpottedOwl,
    AmericanBlackBear,

    // === MIDWATER PELAGIC RIFT CANYON (200-1,500M TWILIGHT DROP-OFF) ===
    // L0: Marine-Snow Mucus-House Urochordates & OMZ Detritivorous Cephalopod
    GiantLarvaceanHouse,
    VampireSquid,
    // L2: Diel Vertical Migration (DVM) Deep Scattering Layer Planktivores
    MyctophidLanternfish,
    SilverHatchetfish,
    PelagicEuphausiidKrill,
    // L3: Mesopelagic Tubular-Eye Ambushers & Siphonophore Drift Curtains
    PacificBarreleye,
    PrayaGiantSiphonophore,
    CockEyedJewelSquid,
    // L4: Deep-Diving Odontocete & Bathypelagic Architeuthid Apex Predators
    DivingSpermWhale,
    ArchiteuthisGiantSquid,

    // === NE PACIFIC ROCKY INTERTIDAL TIDEPOOL & SURGE BENCH ===
    TidepoolHermitCrab,
    BlackKatyChiton,
    CaliforniaMusselBed,
    GooseneckBarnacle,
    GiantGreenAnemone,
    TidepoolSculpin,
    OchreSeaStar,
    BlackOystercatcher,

    // === COLD-SEEP BRINE POOL & METHANE ICE (1,050M UNDERWATER LAKE) ===
    MethaneIceWorm,
    SeepSquatLobster,
    BrineMusselRing,
    AlvinocarisSeepShrimp,
    DeepSeaRedCrab,
    ColdSeepHagfish,
    CutthroatBrineEel,
    SeepGhostChimaera,
}

impl Species {
    pub fn biome(self) -> BiomeKind {
        match self {
            Self::AmanitaFungus
            | Self::MycenaBiolumFungus
            | Self::LeafcutterAnt
            | Self::ThreeToedSloth
            | Self::MorphoButterfly
            | Self::CentralAmericanAgouti
            | Self::LowlandTapir
            | Self::SquirrelMonkey
            | Self::ScarletMacaw
            | Self::KeelBilledToucan
            | Self::BoaConstrictor
            | Self::BlackPanther
            | Self::CanopyJaguar => BiomeKind::NeotropicalRainforest,

            Self::CaliforniaSeaCucumber
            | Self::BatSeaStar
            | Self::NorthernKelpCrab
            | Self::PurpleSeaUrchin
            | Self::RedAbalone
            | Self::BlacksmithDamselfish
            | Self::SeaOtter
            | Self::CaliforniaSheephead
            | Self::GaribaldiDamselfish
            | Self::GiantPacificOctopus
            | Self::PacificHarborSeal
            | Self::LeopardShark
            | Self::PacificWolfEel => BiomeKind::PacificKelpForest,

            Self::SignalCrayfish
            | Self::PumpkinseedSunfish
            | Self::GoldenShiner
            | Self::BlueDasherDragonfly
            | Self::MallardDuck
            | Self::BeltedKingfisher
            | Self::RainbowTrout
            | Self::LargemouthBass
            | Self::WesternPaintedTurtle
            | Self::AmericanBullfrog
            | Self::Osprey
            | Self::CommonLoon
            | Self::GreatBlueHeron
            | Self::NorthernPike => BiomeKind::TemperateFreshwaterLake,

            Self::MunidopsisSquatLobster
            | Self::KiwaYetiCrab
            | Self::RimicarisVentShrimp
            | Self::ScalyFootSnail
            | Self::ZoarcidVentEelpout
            | Self::AtollaCrownJelly
            | Self::BiolumCombJelly
            | Self::AbyssalGhostShark
            | Self::BluntnoseSixgillShark
            | Self::VulcanoctopusVent => BiomeKind::DeepSeaHydrothermalVent,

            Self::PacificBananaSlug
            | Self::GoldenChanterelle
            | Self::CasedCaddisflyLarva
            | Self::SpawningCohoSalmon
            | Self::DouglasSquirrel
            | Self::AmericanDipper
            | Self::PacificGiantSalamander
            | Self::VariedThrush
            | Self::NorthernSpottedOwl
            | Self::AmericanBlackBear => BiomeKind::PnwOldGrowthSalmonCreek,

            Self::GiantLarvaceanHouse
            | Self::VampireSquid
            | Self::MyctophidLanternfish
            | Self::SilverHatchetfish
            | Self::PelagicEuphausiidKrill
            | Self::PacificBarreleye
            | Self::PrayaGiantSiphonophore
            | Self::CockEyedJewelSquid
            | Self::DivingSpermWhale
            | Self::ArchiteuthisGiantSquid => BiomeKind::MidwaterPelagicRift,

            Self::TidepoolHermitCrab
            | Self::BlackKatyChiton
            | Self::CaliforniaMusselBed
            | Self::GooseneckBarnacle
            | Self::GiantGreenAnemone
            | Self::TidepoolSculpin
            | Self::OchreSeaStar
            | Self::BlackOystercatcher => BiomeKind::RockyIntertidalTidepool,

            Self::MethaneIceWorm
            | Self::SeepSquatLobster
            | Self::BrineMusselRing
            | Self::AlvinocarisSeepShrimp
            | Self::DeepSeaRedCrab
            | Self::ColdSeepHagfish
            | Self::CutthroatBrineEel
            | Self::SeepGhostChimaera => BiomeKind::ColdSeepBrinePool,

            _ => BiomeKind::IndoPacificCoralReef,
        }
    }

    pub fn common_name(self) -> &'static str {
        match self {
            Self::AmanitaFungus => "Saprotrophic Forest Mushroom",
            Self::MycenaBiolumFungus => "Bioluminescent Mycena Fungus",
            Self::LeafcutterAnt => "Leafcutter Ant (Atta)",
            Self::ThreeToedSloth => "Brown-Throated Three-Toed Sloth",
            Self::MorphoButterfly => "Blue Morpho / Heliconius Butterfly",
            Self::CentralAmericanAgouti => "Central American Agouti",
            Self::LowlandTapir => "South American Lowland Tapir",
            Self::SquirrelMonkey => "Common Squirrel Monkey",
            Self::ScarletMacaw => "Scarlet Macaw",
            Self::KeelBilledToucan => "Toco / Keel-Billed Toucan",
            Self::BoaConstrictor => "Red-Tailed Boa Constrictor",
            Self::BlackPanther => "Melanistic Black Panther",
            Self::CanopyJaguar => "Spotted Canopy Jaguar",
            Self::CaliforniaSeaCucumber => "California Giant Sea Cucumber",
            Self::BatSeaStar => "Webbed Bat Star",
            Self::NorthernKelpCrab => "Northern Shield Kelp Crab",
            Self::PurpleSeaUrchin => "Pacific Purple Sea Urchin",
            Self::RedAbalone => "Pacific Red Abalone",
            Self::BlacksmithDamselfish => "Blacksmith Planktivore School",
            Self::SeaOtter => "Sea Otter",
            Self::CaliforniaSheephead => "California Sheephead Wrasse",
            Self::GaribaldiDamselfish => "Garibaldi Reef Damselfish",
            Self::GiantPacificOctopus => "Giant Pacific Octopus",
            Self::PacificHarborSeal => "Pacific Harbor Seal",
            Self::LeopardShark => "Pacific Leopard Shark",
            Self::PacificWolfEel => "Pacific Wolf Eel",
            Self::PacificCleanerShrimp => "Scarlet Skunk Cleaner Shrimp",
            Self::BlueLinckiaStar => "Cobalt Linckia Sea Star",
            Self::ReefHermitCrab => "Electric-Blue Banded Hermit Crab",
            Self::GreenSeaTurtle => "Indo-Pacific Green Sea Turtle",
            Self::SteepheadParrotfish => "Turquoise Steephead Parrotfish",
            Self::BlueGreenChromis => "Blue-Green Chromis Coral Cloud",
            Self::GiantTridacnaClam => "Giant Fluted Tridacna Clam",
            Self::OcellarisClownfish => "Ocellaris Anemone Clownfish",
            Self::MoorishIdol => "Moorish Idol",
            Self::EmperorAngelfish => "Emperor Reef Angelfish",
            Self::BluespottedStingray => "Bluespotted Ribbontail Stingray",
            Self::BlacktipReefShark => "Blacktip Reef Shark",
            Self::GiantMorayEel => "Giant Reef Moray Eel",
            Self::RedLionfish => "Indo-Pacific Red Lionfish",
            Self::SignalCrayfish => "Pacific Signal Crayfish",
            Self::PumpkinseedSunfish => "Pumpkinseed Sunfish",
            Self::GoldenShiner => "Golden Shiner School",
            Self::BlueDasherDragonfly => "Blue Dasher Dragonfly",
            Self::MallardDuck => "Mallard Duck",
            Self::BeltedKingfisher => "Belted Kingfisher",
            Self::RainbowTrout => "Native Rainbow Trout",
            Self::LargemouthBass => "Largemouth Bass",
            Self::WesternPaintedTurtle => "Western Painted Turtle",
            Self::AmericanBullfrog => "American Bullfrog",
            Self::Osprey => "Osprey (Sea Hawk)",
            Self::CommonLoon => "Common Loon",
            Self::GreatBlueHeron => "Great Blue Heron",
            Self::NorthernPike => "Northern Pike",
            Self::MunidopsisSquatLobster => "Galatheid Vent Squat Lobster",
            Self::KiwaYetiCrab => "Hairy-Clawed Yeti Crab",
            Self::RimicarisVentShrimp => "Eyeless Chimney Vent Shrimp",
            Self::ScalyFootSnail => "Iron-Armored Scaly-Foot Snail",
            Self::ZoarcidVentEelpout => "Hydrothermal Vent Eelpout",
            Self::AtollaCrownJelly => "Bioluminescent Atolla Crown Jelly",
            Self::BiolumCombJelly => "Iridescent Abyssal Comb Jelly",
            Self::AbyssalGhostShark => "Abyssal Ghost Shark (Chimaera)",
            Self::BluntnoseSixgillShark => "Deep-Sea Bluntnose Sixgill Shark",
            Self::VulcanoctopusVent => "Hydrothermal Vent Octopus",
            Self::PacificBananaSlug => "Pacific Banana Slug",
            Self::GoldenChanterelle => "Pacific Golden Chanterelle",
            Self::CasedCaddisflyLarva => "Pebble-Cased Caddisfly Larva",
            Self::SpawningCohoSalmon => "Spawning Coho Salmon",
            Self::DouglasSquirrel => "Douglas's Chickaree Squirrel",
            Self::AmericanDipper => "American Dipper (Water Ouzel)",
            Self::PacificGiantSalamander => "Pacific Giant Salamander",
            Self::VariedThrush => "Pacific Varied Thrush",
            Self::NorthernSpottedOwl => "Northern Spotted Owl",
            Self::AmericanBlackBear => "Olympic / Cascade Black Bear",
            Self::GiantLarvaceanHouse => "Giant Larvacean & Mucus House",
            Self::VampireSquid => "Vampire Squid",
            Self::MyctophidLanternfish => "Northern Lampfish (Myctophid)",
            Self::SilverHatchetfish => "Giant Silver Hatchetfish",
            Self::PelagicEuphausiidKrill => "North Pacific Pelagic Krill",
            Self::PacificBarreleye => "Pacific Barreleye (Spookfish)",
            Self::PrayaGiantSiphonophore => "Giant Pelagic Siphonophore",
            Self::CockEyedJewelSquid => "Cock-Eyed Strawberry Squid",
            Self::DivingSpermWhale => "Deep-Diving Sperm Whale",
            Self::ArchiteuthisGiantSquid => "Bathypelagic Giant Squid",
            Self::TidepoolHermitCrab => "Blueband Tidepool Hermit Crab",
            Self::BlackKatyChiton => "Black Katy Leather Chiton",
            Self::CaliforniaMusselBed => "California Mussel Bed",
            Self::GooseneckBarnacle => "Pacific Gooseneck Barnacle",
            Self::GiantGreenAnemone => "Giant Green Surf Anemone",
            Self::TidepoolSculpin => "Tidepool Sculpin",
            Self::OchreSeaStar => "Ochre Keystone Sea Star",
            Self::BlackOystercatcher => "Black Oystercatcher",
            Self::MethaneIceWorm => "Methane Hydrate Ice Worm",
            Self::SeepSquatLobster => "Cold-Seep Squat Lobster",
            Self::BrineMusselRing => "Brine-Edge Methanotrophic Mussel",
            Self::AlvinocarisSeepShrimp => "Alvinocarid Methane-Seep Shrimp",
            Self::DeepSeaRedCrab => "Deep-Sea Red Crab",
            Self::ColdSeepHagfish => "Pacific Cold-Seep Hagfish",
            Self::CutthroatBrineEel => "Synaphobranchid Cutthroat Eel",
            Self::SeepGhostChimaera => "Cold-Seep Ghost Chimaera",
        }
    }

    pub fn scientific_name(self) -> &'static str {
        match self {
            Self::AmanitaFungus => "Amanita muscaria var. tropica",
            Self::MycenaBiolumFungus => "Mycena chlorophos",
            Self::LeafcutterAnt => "Atta cephalotes",
            Self::ThreeToedSloth => "Bradypus variegatus",
            Self::MorphoButterfly => "Morpho menelaus",
            Self::CentralAmericanAgouti => "Dasyprocta punctata",
            Self::LowlandTapir => "Tapirus terrestris",
            Self::SquirrelMonkey => "Saimiri sciureus",
            Self::ScarletMacaw => "Ara macao",
            Self::KeelBilledToucan => "Ramphastos toco",
            Self::BoaConstrictor => "Boa constrictor constrictor",
            Self::BlackPanther | Self::CanopyJaguar => "Panthera onca",
            Self::CaliforniaSeaCucumber => "Apostichopus californicus",
            Self::BatSeaStar => "Patiria miniata",
            Self::NorthernKelpCrab => "Pugettia producta",
            Self::PurpleSeaUrchin => "Strongylocentrotus purpuratus",
            Self::RedAbalone => "Haliotis rufescens",
            Self::BlacksmithDamselfish => "Chromis punctipinnis",
            Self::SeaOtter => "Enhydra lutris",
            Self::CaliforniaSheephead => "Semicossyphus pulcher",
            Self::GaribaldiDamselfish => "Hypsypops rubicundus",
            Self::GiantPacificOctopus => "Enteroctopus dofleini",
            Self::PacificHarborSeal => "Phoca vitulina",
            Self::LeopardShark => "Triakis semifasciata",
            Self::PacificWolfEel => "Anarrhichthys ocellatus",
            Self::PacificCleanerShrimp => "Lysmata amboinensis",
            Self::BlueLinckiaStar => "Linckia laevigata",
            Self::ReefHermitCrab => "Calcinus elegans",
            Self::GreenSeaTurtle => "Chelonia mydas",
            Self::SteepheadParrotfish => "Chlorurus microrhinos",
            Self::BlueGreenChromis => "Chromis viridis",
            Self::GiantTridacnaClam => "Tridacna gigas",
            Self::OcellarisClownfish => "Amphiprion ocellaris",
            Self::MoorishIdol => "Zanclus cornutus",
            Self::EmperorAngelfish => "Pomacanthus imperator",
            Self::BluespottedStingray => "Taeniura lymma",
            Self::BlacktipReefShark => "Carcharhinus melanopterus",
            Self::GiantMorayEel => "Gymnothorax javanicus",
            Self::RedLionfish => "Pterois volitans",
            Self::SignalCrayfish => "Pacifastacus leniusculus",
            Self::PumpkinseedSunfish => "Lepomis gibbosus",
            Self::GoldenShiner => "Notemigonus crysoleucas",
            Self::BlueDasherDragonfly => "Pachydiplax longipennis",
            Self::MallardDuck => "Anas platyrhynchos",
            Self::BeltedKingfisher => "Megaceryle alcyon",
            Self::RainbowTrout => "Oncorhynchus mykiss",
            Self::LargemouthBass => "Micropterus salmoides",
            Self::WesternPaintedTurtle => "Chrysemys picta bellii",
            Self::AmericanBullfrog => "Lithobates catesbeianus",
            Self::Osprey => "Pandion haliaetus",
            Self::CommonLoon => "Gavia immer",
            Self::GreatBlueHeron => "Ardea herodias",
            Self::NorthernPike => "Esox lucius",
            Self::MunidopsisSquatLobster => "Munidopsis subsquamosa",
            Self::KiwaYetiCrab => "Kiwa hirsuta",
            Self::RimicarisVentShrimp => "Rimicaris exoculata",
            Self::ScalyFootSnail => "Chrysomallon squamiferum",
            Self::ZoarcidVentEelpout => "Thermarces cerberus",
            Self::AtollaCrownJelly => "Atolla wyvillei",
            Self::BiolumCombJelly => "Beroe abyssicola",
            Self::AbyssalGhostShark => "Hydrolagus colliei",
            Self::BluntnoseSixgillShark => "Hexanchus griseus",
            Self::VulcanoctopusVent => "Vulcanoctopus hydrothermalis",
            Self::PacificBananaSlug => "Ariolimax columbianus",
            Self::GoldenChanterelle => "Cantharellus formosus",
            Self::CasedCaddisflyLarva => "Dicosmoecus gilvipes",
            Self::SpawningCohoSalmon => "Oncorhynchus kisutch",
            Self::DouglasSquirrel => "Tamiasciurus douglasii",
            Self::AmericanDipper => "Cinclus mexicanus",
            Self::PacificGiantSalamander => "Dicamptodon tenebrosus",
            Self::VariedThrush => "Ixoreus naevius",
            Self::NorthernSpottedOwl => "Strix occidentalis caurina",
            Self::AmericanBlackBear => "Ursus americanus",
            Self::GiantLarvaceanHouse => "Bathochordaeus charon",
            Self::VampireSquid => "Vampyroteuthis infernalis",
            Self::MyctophidLanternfish => "Stenobrachius leucopsarus",
            Self::SilverHatchetfish => "Argyropelecus gigas",
            Self::PelagicEuphausiidKrill => "Euphausia pacifica",
            Self::PacificBarreleye => "Macropinna microstoma",
            Self::PrayaGiantSiphonophore => "Praya dubia",
            Self::CockEyedJewelSquid => "Histioteuthis heteropsis",
            Self::DivingSpermWhale => "Physeter macrocephalus",
            Self::ArchiteuthisGiantSquid => "Architeuthis dux",
            Self::TidepoolHermitCrab => "Pagurus samuelis",
            Self::BlackKatyChiton => "Katharina tunicata",
            Self::CaliforniaMusselBed => "Mytilus californianus",
            Self::GooseneckBarnacle => "Pollicipes polymerus",
            Self::GiantGreenAnemone => "Anthopleura xanthogrammica",
            Self::TidepoolSculpin => "Oligocottus maculosus",
            Self::OchreSeaStar => "Pisaster ochraceus",
            Self::BlackOystercatcher => "Haematopus bachmani",
            Self::MethaneIceWorm => "Sirsoe methanicola",
            Self::SeepSquatLobster => "Munidopsis geyeri",
            Self::BrineMusselRing => "Bathymodiolus childressi",
            Self::AlvinocarisSeepShrimp => "Alvinocaris methanophila",
            Self::DeepSeaRedCrab => "Chaceon quinquedens",
            Self::ColdSeepHagfish => "Eptatretus stoutii",
            Self::CutthroatBrineEel => "Synaphobranchus kaupii",
            Self::SeepGhostChimaera => "Hydrolagus alberti",
        }
    }

    pub fn stratum(self) -> TrophicStratum {
        match self {
            Self::AmanitaFungus
            | Self::MycenaBiolumFungus
            | Self::LeafcutterAnt
            | Self::CaliforniaSeaCucumber
            | Self::BatSeaStar
            | Self::NorthernKelpCrab
            | Self::PacificCleanerShrimp
            | Self::BlueLinckiaStar
            | Self::ReefHermitCrab
            | Self::SignalCrayfish
            | Self::MunidopsisSquatLobster
            | Self::PacificBananaSlug
            | Self::GoldenChanterelle
            | Self::CasedCaddisflyLarva
            | Self::GiantLarvaceanHouse
            | Self::VampireSquid
            | Self::TidepoolHermitCrab
            | Self::BlackKatyChiton
            | Self::MethaneIceWorm
            | Self::SeepSquatLobster => TrophicStratum::Decomposers,

            Self::ThreeToedSloth
            | Self::MorphoButterfly
            | Self::CentralAmericanAgouti
            | Self::LowlandTapir
            | Self::PurpleSeaUrchin
            | Self::RedAbalone
            | Self::BlacksmithDamselfish
            | Self::GreenSeaTurtle
            | Self::SteepheadParrotfish
            | Self::BlueGreenChromis
            | Self::GiantTridacnaClam
            | Self::PumpkinseedSunfish
            | Self::GoldenShiner
            | Self::BlueDasherDragonfly
            | Self::KiwaYetiCrab
            | Self::RimicarisVentShrimp
            | Self::ScalyFootSnail
            | Self::SpawningCohoSalmon
            | Self::DouglasSquirrel
            | Self::MyctophidLanternfish
            | Self::SilverHatchetfish
            | Self::PelagicEuphausiidKrill
            | Self::CaliforniaMusselBed
            | Self::GooseneckBarnacle
            | Self::BrineMusselRing
            | Self::AlvinocarisSeepShrimp => TrophicStratum::PrimaryConsumers,

            Self::SquirrelMonkey
            | Self::ScarletMacaw
            | Self::KeelBilledToucan
            | Self::SeaOtter
            | Self::CaliforniaSheephead
            | Self::GaribaldiDamselfish
            | Self::GiantPacificOctopus
            | Self::OcellarisClownfish
            | Self::MoorishIdol
            | Self::EmperorAngelfish
            | Self::BluespottedStingray
            | Self::MallardDuck
            | Self::BeltedKingfisher
            | Self::RainbowTrout
            | Self::LargemouthBass
            | Self::WesternPaintedTurtle
            | Self::AmericanBullfrog
            | Self::ZoarcidVentEelpout
            | Self::AtollaCrownJelly
            | Self::BiolumCombJelly
            | Self::AmericanDipper
            | Self::PacificGiantSalamander
            | Self::VariedThrush
            | Self::PacificBarreleye
            | Self::PrayaGiantSiphonophore
            | Self::CockEyedJewelSquid
            | Self::GiantGreenAnemone
            | Self::TidepoolSculpin
            | Self::DeepSeaRedCrab
            | Self::ColdSeepHagfish => TrophicStratum::SecondaryConsumers,

            Self::BoaConstrictor
            | Self::BlackPanther
            | Self::CanopyJaguar
            | Self::PacificHarborSeal
            | Self::LeopardShark
            | Self::PacificWolfEel
            | Self::BlacktipReefShark
            | Self::GiantMorayEel
            | Self::RedLionfish
            | Self::Osprey
            | Self::CommonLoon
            | Self::GreatBlueHeron
            | Self::NorthernPike
            | Self::AbyssalGhostShark
            | Self::BluntnoseSixgillShark
            | Self::VulcanoctopusVent
            | Self::NorthernSpottedOwl
            | Self::AmericanBlackBear
            | Self::DivingSpermWhale
            | Self::ArchiteuthisGiantSquid
            | Self::OchreSeaStar
            | Self::BlackOystercatcher
            | Self::CutthroatBrineEel
            | Self::SeepGhostChimaera => TrophicStratum::TertiaryCarnivores,
        }
    }

    pub fn typical_mass_kg(self) -> f32 {
        match self {
            Self::AmanitaFungus | Self::MycenaBiolumFungus => 0.04,
            Self::LeafcutterAnt => 0.000_015,
            Self::ThreeToedSloth => 4.2,
            Self::MorphoButterfly => 0.000_4,
            Self::CentralAmericanAgouti => 3.1,
            Self::LowlandTapir => 210.0,
            Self::SquirrelMonkey => 0.92,
            Self::ScarletMacaw => 1.05,
            Self::KeelBilledToucan => 0.62,
            Self::BoaConstrictor => 14.5,
            Self::BlackPanther => 68.0,
            Self::CanopyJaguar => 76.0,
            Self::CaliforniaSeaCucumber => 0.85,
            Self::BatSeaStar => 0.28,
            Self::NorthernKelpCrab => 0.22,
            Self::PurpleSeaUrchin => 0.14,
            Self::RedAbalone => 1.45,
            Self::BlacksmithDamselfish => 0.09,
            Self::SeaOtter => 29.0,
            Self::CaliforniaSheephead => 6.8,
            Self::GaribaldiDamselfish => 0.48,
            Self::GiantPacificOctopus => 24.0,
            Self::PacificHarborSeal => 88.0,
            Self::LeopardShark => 18.5,
            Self::PacificWolfEel => 14.0,
            Self::PacificCleanerShrimp => 0.008,
            Self::BlueLinckiaStar => 0.25,
            Self::ReefHermitCrab => 0.06,
            Self::GreenSeaTurtle => 115.0,
            Self::SteepheadParrotfish => 4.8,
            Self::BlueGreenChromis => 0.025,
            Self::GiantTridacnaClam => 95.0,
            Self::OcellarisClownfish => 0.035,
            Self::MoorishIdol => 0.22,
            Self::EmperorAngelfish => 1.15,
            Self::BluespottedStingray => 4.5,
            Self::BlacktipReefShark => 24.0,
            Self::GiantMorayEel => 22.0,
            Self::RedLionfish => 1.1,
            Self::SignalCrayfish => 0.085,
            Self::PumpkinseedSunfish => 0.22,
            Self::GoldenShiner => 0.045,
            Self::BlueDasherDragonfly => 0.000_6,
            Self::MallardDuck => 1.15,
            Self::BeltedKingfisher => 0.15,
            Self::RainbowTrout => 1.85,
            Self::LargemouthBass => 2.40,
            Self::WesternPaintedTurtle => 0.55,
            Self::AmericanBullfrog => 0.48,
            Self::Osprey => 1.65,
            Self::CommonLoon => 4.20,
            Self::GreatBlueHeron => 2.35,
            Self::NorthernPike => 5.80,
            Self::MunidopsisSquatLobster => 0.045,
            Self::KiwaYetiCrab => 0.14,
            Self::RimicarisVentShrimp => 0.006,
            Self::ScalyFootSnail => 0.032,
            Self::ZoarcidVentEelpout => 0.28,
            Self::AtollaCrownJelly => 0.18,
            Self::BiolumCombJelly => 0.09,
            Self::AbyssalGhostShark => 2.40,
            Self::BluntnoseSixgillShark => 380.0,
            Self::VulcanoctopusVent => 0.45,
            Self::PacificBananaSlug => 0.11,
            Self::GoldenChanterelle => 0.06,
            Self::CasedCaddisflyLarva => 0.002,
            Self::SpawningCohoSalmon => 4.80,
            Self::DouglasSquirrel => 0.24,
            Self::AmericanDipper => 0.058,
            Self::PacificGiantSalamander => 0.14,
            Self::VariedThrush => 0.082,
            Self::NorthernSpottedOwl => 0.64,
            Self::AmericanBlackBear => 135.0,
            Self::GiantLarvaceanHouse => 0.045,
            Self::VampireSquid => 0.55,
            Self::MyctophidLanternfish => 0.018,
            Self::SilverHatchetfish => 0.024,
            Self::PelagicEuphausiidKrill => 0.002,
            Self::PacificBarreleye => 0.18,
            Self::PrayaGiantSiphonophore => 2.80,
            Self::CockEyedJewelSquid => 0.42,
            Self::DivingSpermWhale => 34000.0,
            Self::ArchiteuthisGiantSquid => 240.0,
            Self::TidepoolHermitCrab => 0.015,
            Self::BlackKatyChiton => 0.035,
            Self::CaliforniaMusselBed => 0.25,
            Self::GooseneckBarnacle => 0.08,
            Self::GiantGreenAnemone => 0.32,
            Self::TidepoolSculpin => 0.028,
            Self::OchreSeaStar => 0.85,
            Self::BlackOystercatcher => 0.56,
            Self::MethaneIceWorm => 0.008,
            Self::SeepSquatLobster => 0.065,
            Self::BrineMusselRing => 0.28,
            Self::AlvinocarisSeepShrimp => 0.012,
            Self::DeepSeaRedCrab => 1.15,
            Self::ColdSeepHagfish => 0.42,
            Self::CutthroatBrineEel => 0.95,
            Self::SeepGhostChimaera => 2.80,
        }
    }

    pub fn is_keystone_species(self) -> bool {
        matches!(
            self,
            Self::OchreSeaStar
                | Self::BrineMusselRing
                | Self::SeaOtter
                | Self::SpawningCohoSalmon
                | Self::NorthernSpottedOwl
                | Self::DivingSpermWhale
                | Self::AbyssalGhostShark
                | Self::NorthernPike
                | Self::Osprey
                | Self::BlacktipReefShark
                | Self::BlackPanther
                | Self::CanopyJaguar
        )

    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BehaviorState {
    // Rainforest
    MonkeyVineSwing { vine_id: usize, swing_time: f32, target_branch_id: usize },
    MonkeyBallisticLeap { target_branch_id: usize, target_t: f32 },
    MonkeyPerchedLookAround { branch_id: usize, branch_t: f32, remaining_sec: f32, scan_seed: f32 },
    MonkeyBranchTraverse { branch_id: usize, branch_t: f32, dir: f32 },
    SnakeCoiledAmbush { branch_id: usize, branch_t: f32, patience_sec: f32, tongue_cooldown: f32 },
    SnakeBallisticStrike { anchor_pos: Vec3, strike_target: Vec3, progress: f32 },
    SnakeBaskingDigest { branch_id: usize, branch_t: f32, remaining_sec: f32 },
    PantherUnderbrushStalk { waypoint_x: f32, waypoint_z: f32, stealth_crouch: f32 },
    PantherShadowPause { remaining_sec: f32, look_yaw: f32 },
    PantherPounce { target_pos: Vec3 },
    JaguarCanopyLounge { branch_id: usize, branch_t: f32, tail_sway_phase: f32, remaining_sec: f32 },
    SlothSuspensoryClimb { branch_id: usize, branch_t: f32, dir: f32 },
    SlothHangingFoliageRest { branch_id: usize, branch_t: f32, remaining_sec: f32 },
    AvianCanopyFlight { target_branch_id: usize, target_t: f32 },
    AvianPerchedForage { branch_id: usize, branch_t: f32, remaining_sec: f32 },
    ButterflyPollinationFlutter { target_plant_id: usize, hover_timer: f32 },
    FloorHerbivoreBrowse { target_x: f32, target_z: f32, gnawing_timer: f32 },
    AntOutwardTrail { patch_pos: Vec3, wandering: f32 },
    AntReturningWithLeaf { nest_pos: Vec3 },
    FungalFruiting { maturity: f32, spore_release_timer: f32 },

    // Kelp Forest & Shared Marine Ethology
    OtterSurfaceRaft { remaining_sec: f32, anchor_x: f32, anchor_z: f32 },
    OtterForageDive { target_reef_pos: Vec3, ascending: bool },
    OtterChestStoneTap { remaining_sec: f32, tap_cooldown: f32 },
    SealKelpGlide { waypoint: Vec3 },
    SealMidwaterLookAround { remaining_sec: f32, scan_seed: f32 },
    SharkReefCruise { waypoint: Vec3 },
    WolfEelCreviceAmbush { den_pos: Vec3, lunge_progress: f32, lunge_target: Vec3 },
    OctopusDenCamouflage { den_pos: Vec3, remaining_sec: f32 },
    OctopusReefJet { target_pos: Vec3 },
    ReefFishForage { waypoint: Vec3, pause_sec: f32 },
    PlanktivoreBoidSchool { school_center: Vec3, phase_seed: f32 },
    BenthicGrazerCreep { target_xz: [f32; 2] },

    // Indo-Pacific Coral Reef Specific Ethology
    SeaTurtleReefFlight { waypoint: Vec3, surfacing_for_breath: bool },
    ClownfishAnemoneSymbiosis { anemone_pos: Vec3, dart_phase: f32 },
    StingraySandUndulate { waypoint: Vec3 },
    SessileMantleFilter { open_fraction: f32 },

    // Temperate Freshwater Lake Split-Level Ethology (Underwater + Surface/Aerial Birds!)
    WaterbirdSurfaceSwim { target_x: f32, target_z: f32, dabble_timer: f32, dive_timer: f32 },
    OspreyAerialSoar { orbit_phase: f32, plunge_timer: f32, plunge_x: f32 },
    HeronShalllowStalk { anchor_pos: Vec3, strike_timer: f32 },
    PikeWeedbedAmbush { hover_pos: Vec3, glide_target: Vec3, gliding: bool },

    // Deep-Sea Hydrothermal Vent Ethology
    GhostSharkPectoralFlight { waypoint: Vec3 },
    YetiCrabBacterialWave {home_pos: Vec3, wave_phase: f32},
    ChimneyShrimpSwarm { chimney_pos: Vec3, swirl_phase: f32 },
    BiolumPelagicPulse { drift_center: Vec3, flash_timer: f32 },

    // PNW Old-Growth & Salmon Creek Ethology
    CohoUpstreamReddAndLeap { redd_pos: Vec3, leap_timer: f32, phase: f32 },
    DipperRiffleSubmergeWalk { rock_perch: Vec3, walk_timer: f32 },
    SpottedOwlCanopyPerchAndGlide { perch_a: Vec3, perch_b: Vec3, glide_timer: f32, at_b: bool },

    // Midwater Pelagic Rift Ethology
    DvmVerticalMigrator { day_y: f32, night_y: f32, home_xz: [f32; 2], phase: f32, dvm_boost: f32 },
    BarreleyeTubularScan { hover_anchor: Vec3, lock_timer: f32 },
    SpermWhaleEcholocationDive { cruise_y: f32, dive_timer: f32, sonar_pulse: f32 },
    ArchiteuthisChasmProwl { home_pos: Vec3, jet_timer: f32, phase: f32 },

    // Tidepool & Cold-Seep Brine Pool Ethology
    SculpinTidepoolHop { pool_a: Vec3, pool_b: Vec3, leap_timer: f32, at_b: bool },
    CutthroatEelBrineCruiser { center: Vec3, skim_timer: f32, phase: f32 },
}

impl BehaviorState {
    pub fn description(&self) -> &'static str {
        match self {
            Self::MonkeyVineSwing { .. } => "Swinging across hanging liana",
            Self::MonkeyBallisticLeap { .. } => "Ballistic canopy leap",
            Self::MonkeyPerchedLookAround { .. } => "Perched & scanning surroundings",
            Self::MonkeyBranchTraverse { .. } => "Scampering along bough",
            Self::SnakeCoiledAmbush { .. } => "Lying in wait (coiled ambush)",
            Self::SnakeBallisticStrike { .. } => "Executing ambush strike",
            Self::SnakeBaskingDigest { .. } => "Coiled post-prandial digestion",
            Self::PantherUnderbrushStalk { .. } => "Stealthily stalking through underbrush",
            Self::PantherShadowPause { .. } => "Motionless shadow observation",
            Self::PantherPounce { .. } => "Low-angle ambush pounce",
            Self::JaguarCanopyLounge { .. } => "Lounging & surveying from upper bough",
            Self::SlothSuspensoryClimb { .. } => "Slow inverted suspensory climb",
            Self::SlothHangingFoliageRest { .. } => "Hanging & digesting canopy leaves",
            Self::AvianCanopyFlight { .. } => "Gliding across mid-canopy",
            Self::AvianPerchedForage { .. } => "Perched berry foraging & preening",
            Self::ButterflyPollinationFlutter { .. } => "Fluttering between understory blooms",
            Self::FloorHerbivoreBrowse { gnawing_timer, .. } => {
                if *gnawing_timer > 0.05 { "Foraging & gnawing fallen fruit" } else { "Ambling through forest-floor ferns" }
            }
            Self::AntOutwardTrail { .. } => "Following chemical foraging trail",
            Self::AntReturningWithLeaf { .. } => "Carrying harvested fragment home",
            Self::FungalFruiting { .. } => "Mineralizing detritus & fruiting",
            Self::OtterSurfaceRaft { .. } => "Floating supine wrapped in surface kelp",
            Self::OtterForageDive { ascending, .. } => {
                if *ascending { "Ascending to canopy with sea urchin" } else { "Diving through kelp to rocky reef" }
            }
            Self::OtterChestStoneTap { .. } => "Tapping sea urchin on chest anvil stone",
            Self::SealKelpGlide { .. } => "Weaving gracefully through kelp stipes",
            Self::SealMidwaterLookAround { .. } => "Hovering in kelp gap & scanning school",
            Self::SharkReefCruise { .. } => "Cruising sinuously along fore-reef",
            Self::WolfEelCreviceAmbush { lunge_progress, .. } => {
                if *lunge_progress > 0.01 { "Lunging from reef overhang for prey" } else { "Peering from reef crevice in wait" }
            }
            Self::OctopusDenCamouflage { .. } => "Camouflaged in basalt den (mottled skin)",
            Self::OctopusReefJet { .. } => "Siphon-jetting across reef (flushed crimson)",
            Self::ReefFishForage { pause_sec, .. } => {
                if *pause_sec > 0.1 { "Grazing coral turf & bioeroding sand" } else { "Rowing across coral heads" }
            }
            Self::PlanktivoreBoidSchool { .. } => "Hovering over Acropora coral shelter",
            Self::BenthicGrazerCreep { .. } => "Tending reef substrate & detritus",
            Self::SeaTurtleReefFlight { surfacing_for_breath, .. } => {
                if *surfacing_for_breath { "Surfacing to breathe at lagoon mirror" } else { "Soaring on hydrofoil foreflippers over reef" }
            }
            Self::ClownfishAnemoneSymbiosis { .. } => "Weaving among host sea anemone tentacles",
            Self::StingraySandUndulate { .. } => "Undulating pectoral disc over carbonate sand",
            Self::SessileMantleFilter { open_fraction } => {
                if *open_fraction < 0.55 { "Retracting zooxanthellate mantle (shadow)" } else { "Photosynthesizing & filter-pumping" }
            }
            Self::WaterbirdSurfaceSwim { dabble_timer, dive_timer, .. } => {
                if *dive_timer > 0.05 { "Pursuit-diving underwater through Vallisneria" }
                else if *dabble_timer > 0.05 { "Tipping tail-up to dabble submerged pondweeds" }
                else { "Paddling across the mirror lake waterline" }
            }
            Self::OspreyAerialSoar { plunge_timer, .. } => {
                if *plunge_timer > 0.05 { "Stooping talons-first into lake surface!" }
                else { "Soaring on thermal currents above lake" }
            }
            Self::HeronShalllowStalk { strike_timer, .. } => {
                if *strike_timer > 0.05 { "Spearing minnow in cattail shallows!" }
                else { "Poised motionless in littoral cattail reeds" }
            }
            Self::PikeWeedbedAmbush { gliding, .. } => {
                if *gliding { "Gliding rapidly across sunken driftwood!" }
                else { "Hovering motionless in eelgrass ambush" }
            }
            Self::GhostSharkPectoralFlight { .. } => "Rowing wing-like pectoral fins past black-smoker plumes",
            Self::YetiCrabBacterialWave { .. } => "Waving hairy chelipeds in warm H2S fluid to farm bacteria",
            Self::ChimneyShrimpSwarm { .. } => "Swarming sulfide spire via dorsal thermal photoreceptors",
            Self::BiolumPelagicPulse { flash_timer, .. } => {
                if *flash_timer > 0.1 { "Flashing rotating sapphire bioluminescent alarm ring!" }
                else { "Drifting through bathypelagic dark with rainbow comb-rows" }
            }
            Self::CohoUpstreamReddAndLeap { leap_timer, .. } => {
                if *leap_timer > 0.05 { "Leaping over boulder weir in spawning colors!" }
                else { "Holding in gravel riffle & digging spawning redd" }
            }
            Self::DipperRiffleSubmergeWalk { walk_timer, .. } => {
                if *walk_timer > 0.05 { "Walking underwater on riffle cobbles for caddisflies!" }
                else { "Bobbing ('dipping') on mossy creek boulder" }
            }
            Self::SpottedOwlCanopyPerchAndGlide { glide_timer, .. } => {
                if *glide_timer > 0.05 { "Gliding silently between old-growth Douglas-fir boughs" }
                else { "Perched in mossy canopy — scanning nurse log below" }
            }
            Self::DvmVerticalMigrator { dvm_boost, .. } => {
                if *dvm_boost > 0.1 { "Ascending in Diel Vertical Migration with ventral photophores!" }
                else { "Counter-illuminating in Deep Scattering Layer against 475nm twilight" }
            }
            Self::BarreleyeTubularScan { lock_timer, .. } => {
                if *lock_timer > 0.05 { "Rotating emerald tubular eyes forward inside transparent dome!" }
                else { "Hovering motionless — scanning silhouettes straight upward" }
            }
            Self::SpermWhaleEcholocationDive { dive_timer, .. } => {
                if *dive_timer > 0.05 { "Sweeping spermaceti sonar click cone into the bathypelagic rift!" }
                else { "Cruising upper twilight canyon rim between deep foraging dives" }
            }
            Self::ArchiteuthisChasmProwl { jet_timer, .. } => {
                if *jet_timer > 0.05 { "Mantle-jetting across the deep rift chasm with clubbed tentacles!" }
                else { "Patrolling bathypelagic canyon wall below twilight boundary" }
            }
            Self::SculpinTidepoolHop { leap_timer, .. } => {
                if *leap_timer > 0.05 { "Ballistically leaping across exposed basalt between tidepools!" }
                else { "Perched on coralline tidepool cobbles with pectoral fins" }
            }
            Self::CutthroatEelBrineCruiser { skim_timer, .. } => {
                if *skim_timer > 0.05 { "Dipping into 120-PSU anoxic brine shoreline & recoiling!" }
                else { "Gliding along hypersaline underwater-lake mussel beach" }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ArticulatedPose {
    pub body_yaw: f32,
    pub body_pitch: f32,
    pub head_yaw: f32,
    pub head_pitch: f32,
    pub limb_phase: f32,
    pub spine_phase: f32,
    pub tail_angle: f32,
    pub wing_phase_amp: [f32; 2],
    pub crouch: f32,
    pub tongue_ext: f32,
    pub carrying_item: bool,
    #[serde(default)]
    pub chromatophore: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organism {
    pub id: u32,
    pub species: Species,
    pub pos: Vec3,
    pub vel: Vec3,
    pub scale: f32,
    pub energy_kcal: f32,
    pub max_energy_kcal: f32,
    pub age_hours: f32,
    pub state: BehaviorState,
    pub pose: ArticulatedPose,
    pub vocal_cooldown_sec: f32,
}

impl Organism {
    pub fn stratum(&self) -> TrophicStratum {
        self.species.stratum()
    }

    pub fn speed_m_s(&self) -> f32 {
        self.vel.length()
    }
}
