BEGIN TRANSACTION;

-- Create bewa_Overview

CREATE TABLE IF NOT EXISTS "bewa_Overview" (
	"InnerId"	TEXT UNIQUE,
	"OuterId"	TEXT,
	"TripDomain"	TEXT,
	"ParticipantGroup"	TEXT,
	"OverallDestination"	TEXT,
	"DepartureDate"	TEXT,
	"ReturnDate"	TEXT,
	"TripDescription"	TEXT,
	"TripLabels"	TEXT CHECK(TripLabels IS NULL OR TripLabels REGEXP '^[^, ]+(, [^, ]+)*$'),
	"MapPins"	TEXT,
	"StartNode"	TEXT,
	"EndNode"	TEXT,
	"PhotoStarttime"	TEXT CHECK("PhotoStarttime" GLOB '[0-9][0-9]:[0-9][0-9]'),
	"PhotoEndtime"	TEXT CHECK("PhotoEndtime" GLOB '[0-9][0-9]:[0-9][0-9]'),
	"PhotoAlbums"	TEXT,
	"CoverPhoto"	TEXT,
	"DocumentationNote"	TEXT,
	PRIMARY KEY("InnerId")
);


-- Create bewb_Events

CREATE TABLE IF NOT EXISTS "bewb_Events" (
	"InnerId"	TEXT,
	"Date"	TEXT,
	"Events"	TEXT,
	"Accommodation"	TEXT,
	"AccommodationCountry"	TEXT,
	"AccommodationCoordinatesAccuracy"	TEXT,
	"AccommodationCoordinates"	TEXT CHECK("AccommodationCoordinates" IS NULL OR "AccommodationCoordinates" GLOB '[-0-9]*[.][0-9]*, [-0-9]*[.][0-9]*'),
	"TravelParticipants"	TEXT,
	"AdditionalNotes"	TEXT,
	"CountriesDuringDay"	TEXT CHECK("CountriesDuringDay" IS NULL OR "CountriesDuringDay" REGEXP '^(([+]{0,1}|[*]{0,2})[a-zA-ZÅÄÖåäö.-]+(?:, ([+]{0,1}|[*]{0,2})[a-zA-ZÅÄÖåäö.-]+)*)$'),
	"OLD_InnerId"	TEXT,
	PRIMARY KEY("InnerId","Date"),
	FOREIGN KEY("InnerId") REFERENCES "bewa_Overview"("InnerId")
);


-- Create bewx_Settings

CREATE TABLE IF NOT EXISTS "bewx_Settings" (
	"AttributeGroup"	TEXT,
	"Attribute"	TEXT UNIQUE,
	"Value"	TEXT
);
INSERT INTO "bewx_Settings" VALUES ('Base','HomeContinent','Europa');
INSERT INTO "bewx_Settings" VALUES ('Base','HomeCountry','Sverige');
INSERT INTO "bewx_Settings" VALUES ('Base','LanguageFile','swedish.json');
INSERT INTO "bewx_Settings" VALUES ('Definition','ContinentCountries','Europa:Albanien:AL
Europa:Andorra:AD
Europa:Belgien:BE
Europa:Bosnien-och-Hercegovina:BA
Europa:Bulgarien:BG
Europa:Cypern:CY
Europa:Cypern-Nordcypern
Europa:Danmark:DK
Europa:Danmark-Färöarna:FO
Europa:Estland:EE
Europa:Finland:FI
Europa:Finland-Åland:AX
Europa:Frankrike:FR
Europa:Georgien:GE
Europa:Grekland:GR
Europa:Irland:IE
Europa:Island:IS
Europa:Italien:IT
Europa:Kosovo:XK
Europa:Kroatien:HR
Europa:Lettland:LV
Europa:Liechtenstein:LI
Europa:Litauen:LT
Europa:Luxemburg:LU
Europa:Malta:MT
Europa:Moldavien:MD
Europa:Moldavien-Transnistrien
Europa:Monaco:MC
Europa:Montenegro:ME
Europa:Nederländerna:NL
Europa:Nordmakedonien:MK
Europa:Norge:NO
Europa:Polen:PL
Europa:Portugal:PT
Europa:Rumänien:RO
Europa:Ryssland:RU
Europa:San-Marino:SM
Europa:Schweiz:CH
Europa:Serbien:RS
Europa:Slovakien:SK
Europa:Slovenien:SI
Europa:Spanien:ES
Europa:Storbritannien:GB
Europa:Storbritannien-Akrotiri-och-Dhekelia
Europa:Storbritannien-Gibraltar:GI
Europa:Storbritannien-Jersey:JE
Europa:Storbritannien-Nordirland:GB
Europa:Sverige:SE
Europa:Tjeckien:CZ
Europa:Tyskland:DE
Europa:Ukraina:UA
Europa:Ungern:HU
Europa:Vatikanstaten:VA
Europa:Vitryssland:BY
Europa:Österrike:AT
Afrika:Algeriet:DZ
Afrika:Angola:AO
Afrika:Benin:BJ
Afrika:Botswana:BW
Afrika:Burkina-Faso:BF
Afrika:Burundi:BI
Afrika:Cabo-Verde:CV
Afrika:Centralafrikanska-republiken:CF
Afrika:Demokratiska-republiken-Kongo:CD
Afrika:Djibouti:DJ
Afrika:Egypten:EG
Afrika:Ekvatorial-Guinea:GQ
Afrika:Elfenbenskusten:CI
Afrika:Eritrea:ER
Afrika:Eswatini:SZ
Afrika:Etiopien:ET
Afrika:Gabon:GA
Afrika:Gambia:GM
Afrika:Ghana:GH
Afrika:Guinea:GN
Afrika:Guinea-Bissau:GW
Afrika:Kamerun:CM
Afrika:Kenya:KE
Afrika:Komorerna:KM
Afrika:Lesotho:LS
Afrika:Liberia:LR
Afrika:Libyen:LY
Afrika:Madagaskar:MG
Afrika:Malawi:MW
Afrika:Mali:ML
Afrika:Marocko:MA
Afrika:Mauretanien:MR
Afrika:Mauritius:MU
Afrika:Moçambique:MZ
Afrika:Namibia:NA
Afrika:Niger:NE
Afrika:Nigeria:NG
Afrika:Republiken-Kongo:CG
Afrika:Rwanda:RW
Afrika:Senegal:SN
Afrika:Seychellerna:SC
Afrika:Sierra-Leone:SL
Afrika:Somalia:SO
Afrika:Sudan:SD
Afrika:Sydafrika:ZA
Afrika:Sydsudan:SS
Afrika:São-Tomé-och-Príncipe:ST
Afrika:Tanzania:TZ
Afrika:Tchad:TD
Afrika:Togo:TG
Afrika:Tunisien:TN
Afrika:Uganda:UG
Afrika:Zambia:ZM
Afrika:Zimbabwe:ZW
Asien:Afghanistan:AF
Asien:Armenien:AM
Asien:Azerbajdzjan:AZ
Asien:Bahrain:BH
Asien:Bangladesh:BD
Asien:Bhutan:BT
Asien:Brunei:BN
Asien:Cypern:CY
Asien:Cypern-Nordcypern
Asien:Filippinerna:PH
Asien:Förenade-Arabemiraten:AE
Asien:Georgien:GE
Asien:Indien:IN
Asien:Indonesien:ID
Asien:Irak:IQ
Asien:Iran:IR
Asien:Israel:IL
Asien:Japan:JP
Asien:Jemen:YE
Asien:Jordanien:JO
Asien:Kambodja:KH
Asien:Kazakstan:KZ
Asien:Kina:CN
Asien:Kirgizistan:KG
Asien:Kuwait:KW
Asien:Laos:LA
Asien:Libanon:LB
Asien:Malaysia:MY
Asien:Maldiverna:MV
Asien:Mongoliet:MN
Asien:Myanmar:MM
Asien:Nepal:NP
Asien:Nordkorea:KP
Asien:Oman:OM
Asien:Pakistan:PK
Asien:Qatar:QA
Asien:Ryssland:RU
Asien:Saudi-Arabien:SA
Asien:Singapore:SG
Asien:Sri-Lanka:LK
Asien:Sydkorea:KR
Asien:Syrien:SY
Asien:Tadzjikistan:TJ
Asien:Taiwan:TW
Asien:Thailand:TH
Asien:Timor-Leste:TL
Asien:Turkiet:TR
Asien:Turkmenistan:TM
Asien:Uzbekistan:UZ
Asien:Vietnam:VN
Nordamerika:Antigua-and-Barbuda:AG
Nordamerika:Bahamas:BS
Nordamerika:Barbados:BB
Nordamerika:Belize:BZ
Nordamerika:Costa-Rica:CR
Nordamerika:Danmark-Grönland:GL
Nordamerika:Dominica:DM
Nordamerika:Dominikanska-Republiken:DO
Nordamerika:El-Salvador:SV
Nordamerika:Grenada:GD
Nordamerika:Guatemala:GT
Nordamerika:Haiti:HT
Nordamerika:Honduras:HN
Nordamerika:Jamaica:JM
Nordamerika:Kanada:CA
Nordamerika:Kuba:CU
Nordamerika:Mexiko:MX
Nordamerika:Nicaragua:NI
Nordamerika:Panama:PA
Nordamerika:Saint-Kitts-and-Nevis:KN
Nordamerika:Saint-Lucia:LC
Nordamerika:Saint-Vincent-and-the-Grenadines:VC
Nordamerika:Trinidad-and-Tobago:TT
Nordamerika:USA:US
Oceanien:Australien:AU
Oceanien:Fiji:FJ
Oceanien:Kiribati:KI
Oceanien:Marshallöarna:MH
Oceanien:Mikronesien:FM
Oceanien:Nauru:NR
Oceanien:Nya-Zeeland:NZ
Oceanien:Palau:PW
Oceanien:Papua-Nya-Guinea:PG
Oceanien:Salomonöarna:SB
Oceanien:Samoa:WS
Oceanien:Tonga:TO
Oceanien:Tuvalu:TV
Oceanien:Vanuatu:VU
Sydamerika:Argentina:AR
Sydamerika:Bolivia:BO
Sydamerika:Brasilien:BR
Sydamerika:Chile:CL
Sydamerika:Colombia:CO
Sydamerika:Ecuador:EC
Sydamerika:Guyana:GY
Sydamerika:Paraguay:PY
Sydamerika:Peru:PE
Sydamerika:Surinam:SR
Sydamerika:Uruguay:UY
Sydamerika:Venezuela:VE');
INSERT INTO "bewx_Settings" VALUES ('Definition','TripDomainColors','Inrikes:#0b5394
Utrikes:#1d655e
Anknytning:#C60C30
Tjänst:#77065d');
INSERT INTO "bewx_Settings" VALUES ('Photos','Immich','Disabled');
INSERT INTO "bewx_Settings" VALUES ('Photos','ImmichApiKey','YOUR_API_KEY');
INSERT INTO "bewx_Settings" VALUES ('Photos','ImmichCoverAlbumId','YOUR_COVER_ALBUM_ID');
INSERT INTO "bewx_Settings" VALUES ('Photos','ImmichUrl','YOUR_IMMICH_URL');
INSERT INTO "bewx_Settings" VALUES ('Other','Dataset','Enabled');
INSERT INTO "bewx_Settings" VALUES ('Other','ExternalMapProvider','https://www.google.com/maps/?q=');

COMMIT;
