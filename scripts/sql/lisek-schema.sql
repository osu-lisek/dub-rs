--
-- PostgreSQL database dump
--

-- Dumped from database version 16.2
-- Dumped by pg_dump version 16.2

-- Started on 2024-06-08 19:18:46

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- TOC entry 5 (class 2615 OID 2200)
-- Name: public; Type: SCHEMA; Schema: -; Owner: dub
--

-- *not* creating schema, since initdb creates it


ALTER SCHEMA public OWNER TO dub;

--
-- TOC entry 3723 (class 0 OID 0)
-- Dependencies: 5
-- Name: SCHEMA public; Type: COMMENT; Schema: -; Owner: dub
--

COMMENT ON SCHEMA public IS '';


--
-- TOC entry 887 (class 1247 OID 16390)
-- Name: PunishmentLevel; Type: TYPE; Schema: public; Owner: dub
--

CREATE TYPE public."PunishmentLevel" AS ENUM (
    'LOW',
    'MEDIUM',
    'CRITICAL'
);


ALTER TYPE public."PunishmentLevel" OWNER TO dub;

--
-- TOC entry 890 (class 1247 OID 16398)
-- Name: PunishmentType; Type: TYPE; Schema: public; Owner: dub
--

CREATE TYPE public."PunishmentType" AS ENUM (
    'TIMEOUT',
    'RESTRICTION'
);


ALTER TYPE public."PunishmentType" OWNER TO dub;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- TOC entry 215 (class 1259 OID 16403)
-- Name: AuthorizedUserApplication; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."AuthorizedUserApplication" (
    id integer NOT NULL,
    "oauthApplicationId" integer NOT NULL,
    scopes text[],
    "userId" integer NOT NULL,
    code text NOT NULL
);


ALTER TABLE public."AuthorizedUserApplication" OWNER TO dub;

--
-- TOC entry 216 (class 1259 OID 16408)
-- Name: AuthorizedUserApplication_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."AuthorizedUserApplication_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."AuthorizedUserApplication_id_seq" OWNER TO dub;

--
-- TOC entry 3725 (class 0 OID 0)
-- Dependencies: 216
-- Name: AuthorizedUserApplication_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."AuthorizedUserApplication_id_seq" OWNED BY public."AuthorizedUserApplication".id;


--
-- TOC entry 217 (class 1259 OID 16409)
-- Name: Beatmap; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Beatmap" (
    id integer NOT NULL,
    title text NOT NULL,
    "titleUnicode" text NOT NULL,
    artist text NOT NULL,
    "artistUnicode" text NOT NULL,
    creator text NOT NULL,
    version text NOT NULL,
    "parentId" integer,
    "beatmapId" integer NOT NULL,
    ar double precision NOT NULL,
    od double precision NOT NULL,
    cs double precision NOT NULL,
    hp double precision NOT NULL,
    stars double precision NOT NULL,
    "gameMode" integer NOT NULL,
    bpm double precision NOT NULL,
    "maxCombo" integer NOT NULL,
    "hitLength" integer NOT NULL,
    "totalLength" integer NOT NULL,
    status integer NOT NULL,
    frozen boolean NOT NULL,
    checksum text NOT NULL,
    "statusReason" text,
    "updatedStatusById" integer DEFAULT '-1'::integer NOT NULL,
    "creatorId" integer DEFAULT '-1'::integer NOT NULL,
    "lastUpdate" timestamp(3) without time zone,
    "lastStatusUpdate" timestamp(3) without time zone
);


ALTER TABLE public."Beatmap" OWNER TO dub;

--
-- TOC entry 218 (class 1259 OID 16416)
-- Name: Beatmap_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."Beatmap_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."Beatmap_id_seq" OWNER TO dub;

--
-- TOC entry 3726 (class 0 OID 0)
-- Dependencies: 218
-- Name: Beatmap_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."Beatmap_id_seq" OWNED BY public."Beatmap".id;


--
-- TOC entry 219 (class 1259 OID 16417)
-- Name: Channel; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Channel" (
    id integer NOT NULL,
    name text NOT NULL,
    description text,
    channel_type text DEFAULT 'public'::text NOT NULL
);


ALTER TABLE public."Channel" OWNER TO dub;

--
-- TOC entry 220 (class 1259 OID 16423)
-- Name: Channel_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."Channel_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."Channel_id_seq" OWNER TO dub;

--
-- TOC entry 3727 (class 0 OID 0)
-- Dependencies: 220
-- Name: Channel_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."Channel_id_seq" OWNED BY public."Channel".id;


--
-- TOC entry 221 (class 1259 OID 16424)
-- Name: GraphEntry; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."GraphEntry" (
    id integer NOT NULL,
    "userId" integer NOT NULL,
    date timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    pp double precision NOT NULL,
    acc double precision NOT NULL,
    "playCount" integer NOT NULL,
    "playTime" integer NOT NULL,
    mode integer NOT NULL,
    rank integer NOT NULL
);


ALTER TABLE public."GraphEntry" OWNER TO dub;

--
-- TOC entry 222 (class 1259 OID 16428)
-- Name: GraphEntry_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."GraphEntry_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."GraphEntry_id_seq" OWNER TO dub;

--
-- TOC entry 3728 (class 0 OID 0)
-- Dependencies: 222
-- Name: GraphEntry_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."GraphEntry_id_seq" OWNED BY public."GraphEntry".id;


--
-- TOC entry 260 (class 1259 OID 16850)
-- Name: Group; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Group" (
    id text NOT NULL,
    name text NOT NULL,
    permissions integer DEFAULT 0 NOT NULL,
    "badgeId" integer
);


ALTER TABLE public."Group" OWNER TO dub;

--
-- TOC entry 223 (class 1259 OID 16429)
-- Name: Hwid; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Hwid" (
    id integer NOT NULL,
    "userId" integer NOT NULL,
    mac text,
    "uniqueId" text,
    "diskId" text
);


ALTER TABLE public."Hwid" OWNER TO dub;

--
-- TOC entry 224 (class 1259 OID 16434)
-- Name: Hwid_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."Hwid_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."Hwid_id_seq" OWNER TO dub;

--
-- TOC entry 3729 (class 0 OID 0)
-- Dependencies: 224
-- Name: Hwid_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."Hwid_id_seq" OWNED BY public."Hwid".id;


--
-- TOC entry 225 (class 1259 OID 16435)
-- Name: Hype; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Hype" (
    id integer NOT NULL,
    "userId" integer NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "beatmapSetId" integer NOT NULL
);


ALTER TABLE public."Hype" OWNER TO dub;

--
-- TOC entry 226 (class 1259 OID 16439)
-- Name: Hype_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."Hype_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."Hype_id_seq" OWNER TO dub;

--
-- TOC entry 3730 (class 0 OID 0)
-- Dependencies: 226
-- Name: Hype_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."Hype_id_seq" OWNED BY public."Hype".id;


--
-- TOC entry 227 (class 1259 OID 16440)
-- Name: Integration; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Integration" (
    id integer NOT NULL,
    name text NOT NULL,
    redirect text NOT NULL
);


ALTER TABLE public."Integration" OWNER TO dub;

--
-- TOC entry 228 (class 1259 OID 16445)
-- Name: Integration_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."Integration_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."Integration_id_seq" OWNER TO dub;

--
-- TOC entry 3731 (class 0 OID 0)
-- Dependencies: 228
-- Name: Integration_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."Integration_id_seq" OWNED BY public."Integration".id;


--
-- TOC entry 229 (class 1259 OID 16446)
-- Name: LinkedIntegration; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."LinkedIntegration" (
    id text NOT NULL,
    "displayName" text NOT NULL,
    "platformId" text NOT NULL,
    "avatarUrl" text NOT NULL,
    "userId" integer NOT NULL,
    "platformType" integer,
    visible boolean DEFAULT true NOT NULL
);


ALTER TABLE public."LinkedIntegration" OWNER TO dub;

--
-- TOC entry 230 (class 1259 OID 16452)
-- Name: Message; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Message" (
    id integer NOT NULL,
    "channelId" integer NOT NULL,
    "userId" integer NOT NULL,
    content text NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public."Message" OWNER TO dub;

--
-- TOC entry 231 (class 1259 OID 16458)
-- Name: Message_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."Message_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."Message_id_seq" OWNER TO dub;

--
-- TOC entry 3732 (class 0 OID 0)
-- Dependencies: 231
-- Name: Message_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."Message_id_seq" OWNED BY public."Message".id;


--
-- TOC entry 232 (class 1259 OID 16459)
-- Name: MultiplayerAction; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."MultiplayerAction" (
    id integer NOT NULL,
    "userId" integer NOT NULL,
    "matchId" integer NOT NULL,
    action integer NOT NULL,
    "time" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "beatmapId" integer,
    data text DEFAULT ''::text NOT NULL
);


ALTER TABLE public."MultiplayerAction" OWNER TO dub;

--
-- TOC entry 233 (class 1259 OID 16466)
-- Name: MultiplayerAction_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."MultiplayerAction_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."MultiplayerAction_id_seq" OWNER TO dub;

--
-- TOC entry 3733 (class 0 OID 0)
-- Dependencies: 233
-- Name: MultiplayerAction_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."MultiplayerAction_id_seq" OWNED BY public."MultiplayerAction".id;


--
-- TOC entry 234 (class 1259 OID 16467)
-- Name: MultiplayerMatch; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."MultiplayerMatch" (
    id integer NOT NULL,
    public boolean DEFAULT true NOT NULL
);


ALTER TABLE public."MultiplayerMatch" OWNER TO dub;

--
-- TOC entry 235 (class 1259 OID 16471)
-- Name: MultiplayerMatch_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."MultiplayerMatch_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."MultiplayerMatch_id_seq" OWNER TO dub;

--
-- TOC entry 3734 (class 0 OID 0)
-- Dependencies: 235
-- Name: MultiplayerMatch_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."MultiplayerMatch_id_seq" OWNED BY public."MultiplayerMatch".id;


--
-- TOC entry 236 (class 1259 OID 16472)
-- Name: MultiplayerParticipant; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."MultiplayerParticipant" (
    id integer NOT NULL,
    "userId" integer NOT NULL,
    "matchId" integer NOT NULL
);


ALTER TABLE public."MultiplayerParticipant" OWNER TO dub;

--
-- TOC entry 237 (class 1259 OID 16475)
-- Name: MultiplayerParticipant_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."MultiplayerParticipant_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."MultiplayerParticipant_id_seq" OWNER TO dub;

--
-- TOC entry 3735 (class 0 OID 0)
-- Dependencies: 237
-- Name: MultiplayerParticipant_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."MultiplayerParticipant_id_seq" OWNED BY public."MultiplayerParticipant".id;


--
-- TOC entry 238 (class 1259 OID 16476)
-- Name: Notifications; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Notifications" (
    id text NOT NULL,
    "userId" integer NOT NULL,
    icon text DEFAULT 'faQuestion'::text,
    "isIconAvatar" boolean DEFAULT false NOT NULL,
    "actionId" text NOT NULL,
    text text NOT NULL,
    seen boolean DEFAULT false NOT NULL
);


ALTER TABLE public."Notifications" OWNER TO dub;

--
-- TOC entry 239 (class 1259 OID 16484)
-- Name: OAuthApplication; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."OAuthApplication" (
    id integer NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    secret text NOT NULL,
    "redirectUrl" text NOT NULL,
    "ownerId" integer NOT NULL,
    "iconHash" text,
    "allowedGrantTypes" text[] DEFAULT ARRAY['authorization_code'::text]
);


ALTER TABLE public."OAuthApplication" OWNER TO dub;

--
-- TOC entry 240 (class 1259 OID 16490)
-- Name: OAuthApplication_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."OAuthApplication_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."OAuthApplication_id_seq" OWNER TO dub;

--
-- TOC entry 3736 (class 0 OID 0)
-- Dependencies: 240
-- Name: OAuthApplication_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."OAuthApplication_id_seq" OWNED BY public."OAuthApplication".id;


--
-- TOC entry 241 (class 1259 OID 16491)
-- Name: PassKeys; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."PassKeys" (
    id integer NOT NULL,
    "userIdHexed" text NOT NULL,
    "userId" integer NOT NULL,
    challange text NOT NULL,
    "credId" text,
    "rawId" text,
    "Type" text,
    "authenticatorAttachment" text
);


ALTER TABLE public."PassKeys" OWNER TO dub;

--
-- TOC entry 242 (class 1259 OID 16496)
-- Name: PassKeys_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."PassKeys_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."PassKeys_id_seq" OWNER TO dub;

--
-- TOC entry 3737 (class 0 OID 0)
-- Dependencies: 242
-- Name: PassKeys_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."PassKeys_id_seq" OWNED BY public."PassKeys".id;


--
-- TOC entry 243 (class 1259 OID 16497)
-- Name: PaymentEntry; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."PaymentEntry" (
    id text NOT NULL,
    "userId" integer NOT NULL,
    "paymentId" integer NOT NULL,
    "paymentSystem" text DEFAULT 'dummy'::text NOT NULL,
    sum integer NOT NULL,
    currency text NOT NULL,
    "finishedAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    until timestamp(3) without time zone NOT NULL
);


ALTER TABLE public."PaymentEntry" OWNER TO dub;

--
-- TOC entry 259 (class 1259 OID 16841)
-- Name: Post; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Post" (
    id text NOT NULL,
    date timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "createdBy" integer DEFAULT '-1'::integer NOT NULL,
    title text NOT NULL,
    subtitle text NOT NULL,
    description text NOT NULL,
    "imageUrl" text NOT NULL
);


ALTER TABLE public."Post" OWNER TO dub;

--
-- TOC entry 244 (class 1259 OID 16504)
-- Name: Punishment; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Punishment" (
    id text NOT NULL,
    date timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    level text NOT NULL,
    "appliedBy" integer NOT NULL,
    "appliedTo" integer NOT NULL,
    "punishmentType" text NOT NULL,
    expires boolean DEFAULT true NOT NULL,
    "expiresAt" timestamp(3) without time zone,
    note text
);


ALTER TABLE public."Punishment" OWNER TO dub;

--
-- TOC entry 245 (class 1259 OID 16511)
-- Name: RelationShips; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."RelationShips" (
    id integer NOT NULL,
    "userId" integer NOT NULL,
    "friendId" integer NOT NULL
);


ALTER TABLE public."RelationShips" OWNER TO dub;

--
-- TOC entry 246 (class 1259 OID 16514)
-- Name: RelationShips_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."RelationShips_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."RelationShips_id_seq" OWNER TO dub;

--
-- TOC entry 3738 (class 0 OID 0)
-- Dependencies: 246
-- Name: RelationShips_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."RelationShips_id_seq" OWNED BY public."RelationShips".id;


--
-- TOC entry 247 (class 1259 OID 16515)
-- Name: Report; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Report" (
    id integer NOT NULL,
    "reportedUser" integer NOT NULL,
    "scoreId" integer,
    "repporterId" integer NOT NULL,
    "reasonId" integer NOT NULL,
    reason text NOT NULL
);


ALTER TABLE public."Report" OWNER TO dub;

--
-- TOC entry 248 (class 1259 OID 16520)
-- Name: Report_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."Report_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."Report_id_seq" OWNER TO dub;

--
-- TOC entry 3739 (class 0 OID 0)
-- Dependencies: 248
-- Name: Report_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."Report_id_seq" OWNED BY public."Report".id;


--
-- TOC entry 249 (class 1259 OID 16521)
-- Name: Score; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."Score" (
    id integer NOT NULL,
    "userId" integer NOT NULL,
    "beatmapChecksum" text NOT NULL,
    "playMode" integer NOT NULL,
    "totalScore" integer NOT NULL,
    "maxCombo" integer NOT NULL,
    count300 integer NOT NULL,
    count100 integer NOT NULL,
    count50 integer NOT NULL,
    "countGeKi" integer NOT NULL,
    "countKatu" integer NOT NULL,
    "countMiss" integer NOT NULL,
    mods integer NOT NULL,
    "isRelax" boolean DEFAULT false NOT NULL,
    perfect boolean NOT NULL,
    status integer NOT NULL,
    performance double precision NOT NULL,
    "submittedAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public."Score" OWNER TO dub;

--
-- TOC entry 250 (class 1259 OID 16528)
-- Name: Score_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."Score_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."Score_id_seq" OWNER TO dub;

--
-- TOC entry 3740 (class 0 OID 0)
-- Dependencies: 250
-- Name: Score_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."Score_id_seq" OWNED BY public."Score".id;


--
-- TOC entry 251 (class 1259 OID 16529)
-- Name: User; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."User" (
    id integer NOT NULL,
    username text NOT NULL,
    "usernameSafe" text DEFAULT ''::text NOT NULL,
    password text,
    email text,
    flags integer DEFAULT 0 NOT NULL,
    permissions integer DEFAULT 0 NOT NULL,
    country text DEFAULT 'XX'::text NOT NULL,
    "forgotPassword" boolean DEFAULT false NOT NULL,
    "lastSeen" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "createdAt" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    notes text DEFAULT ''::text,
    "hypeRemaining" integer DEFAULT 6 NOT NULL,
    "donorUntil" timestamp(3) without time zone,
    "backgroundUrl" text,
    "ircPassword" text,
    "countryChangesRemaining" integer DEFAULT 0 NOT NULL,
    "oldUsernames" text[],
    "usernameChangesRemaining" integer DEFAULT 0 NOT NULL,
    "lastHype" timestamp(3) without time zone DEFAULT CURRENT_TIMESTAMP,
    "userpageContent" text DEFAULT ''::text NOT NULL,
    coins integer DEFAULT 0 NOT NULL
);


ALTER TABLE public."User" OWNER TO dub;

--
-- TOC entry 252 (class 1259 OID 16548)
-- Name: UserBadge; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."UserBadge" (
    id integer NOT NULL,
    name text NOT NULL,
    icon text NOT NULL,
    color text NOT NULL,
    "userId" integer NOT NULL
);


ALTER TABLE public."UserBadge" OWNER TO dub;

--
-- TOC entry 253 (class 1259 OID 16553)
-- Name: UserBadge_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."UserBadge_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."UserBadge_id_seq" OWNER TO dub;

--
-- TOC entry 3741 (class 0 OID 0)
-- Dependencies: 253
-- Name: UserBadge_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."UserBadge_id_seq" OWNED BY public."UserBadge".id;


--
-- TOC entry 262 (class 1259 OID 16859)
-- Name: UserGroup; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."UserGroup" (
    id integer NOT NULL,
    "userId" integer NOT NULL,
    "groupId" integer NOT NULL
);


ALTER TABLE public."UserGroup" OWNER TO dub;

--
-- TOC entry 261 (class 1259 OID 16858)
-- Name: UserGroup_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."UserGroup_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."UserGroup_id_seq" OWNER TO dub;

--
-- TOC entry 3742 (class 0 OID 0)
-- Dependencies: 261
-- Name: UserGroup_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."UserGroup_id_seq" OWNED BY public."UserGroup".id;


--
-- TOC entry 254 (class 1259 OID 16554)
-- Name: UserStats; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."UserStats" (
    id integer NOT NULL,
    "userId" integer NOT NULL,
    "rankedScoreStd" bigint DEFAULT 0 NOT NULL,
    "rankedScoreTaiko" bigint DEFAULT 0 NOT NULL,
    "rankedScoreCtb" bigint DEFAULT 0 NOT NULL,
    "rankedScoreMania" bigint DEFAULT 0 NOT NULL,
    "rankedScoreRx" bigint DEFAULT 0 NOT NULL,
    "totalScoreStd" bigint DEFAULT 0 NOT NULL,
    "totalScoreTaiko" bigint DEFAULT 0 NOT NULL,
    "totalScoreCtb" bigint DEFAULT 0 NOT NULL,
    "totalScoreMania" bigint DEFAULT 0 NOT NULL,
    "totalScoreRx" bigint DEFAULT 0 NOT NULL,
    "ppStd" double precision DEFAULT 0 NOT NULL,
    "ppTaiko" double precision DEFAULT 0 NOT NULL,
    "ppCtb" double precision DEFAULT 0 NOT NULL,
    "ppMania" double precision DEFAULT 0 NOT NULL,
    "ppRx" double precision DEFAULT 0 NOT NULL,
    "avgAccStd" double precision DEFAULT 0 NOT NULL,
    "avgAccTaiko" double precision DEFAULT 0 NOT NULL,
    "avgAccCtb" double precision DEFAULT 0 NOT NULL,
    "avgAccMania" double precision DEFAULT 0 NOT NULL,
    "avgAccRx" double precision DEFAULT 0 NOT NULL,
    "playCountStd" integer DEFAULT 0 NOT NULL,
    "playCountTaiko" integer DEFAULT 0 NOT NULL,
    "playCountCtb" integer DEFAULT 0 NOT NULL,
    "playCountMania" integer DEFAULT 0 NOT NULL,
    "playCountRx" integer DEFAULT 0 NOT NULL,
    "playTimeStd" integer DEFAULT 0 NOT NULL,
    "playTimeTaiko" integer DEFAULT 0 NOT NULL,
    "playTimeCtb" integer DEFAULT 0 NOT NULL,
    "playTimeMania" integer DEFAULT 0 NOT NULL,
    "playTimeRx" integer DEFAULT 0 NOT NULL,
    "maxComboStd" integer DEFAULT 0 NOT NULL,
    "maxComboTaiko" integer DEFAULT 0 NOT NULL,
    "maxComboCtb" integer DEFAULT 0 NOT NULL,
    "maxComboMania" integer DEFAULT 0 NOT NULL,
    "maxComboRx" integer DEFAULT 0 NOT NULL,
    "replaysWatchedStd" integer DEFAULT 0 NOT NULL,
    "replaysWatchedTaiko" integer DEFAULT 0 NOT NULL,
    "replaysWatchedCtb" integer DEFAULT 0 NOT NULL,
    "replaysWatchedMania" integer DEFAULT 0 NOT NULL,
    "replaysWatchedRx" integer DEFAULT 0 NOT NULL,
    "hitsStd" integer DEFAULT 0 NOT NULL,
    "hitsTaiko" integer DEFAULT 0 NOT NULL,
    "hitsCtb" integer DEFAULT 0 NOT NULL,
    "hitsMania" integer DEFAULT 0 NOT NULL,
    "hitsRx" integer DEFAULT 0 NOT NULL
);


ALTER TABLE public."UserStats" OWNER TO dub;

--
-- TOC entry 255 (class 1259 OID 16602)
-- Name: UserStats_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."UserStats_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."UserStats_id_seq" OWNER TO dub;

--
-- TOC entry 3743 (class 0 OID 0)
-- Dependencies: 255
-- Name: UserStats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."UserStats_id_seq" OWNED BY public."UserStats".id;


--
-- TOC entry 256 (class 1259 OID 16603)
-- Name: User_id_seq; Type: SEQUENCE; Schema: public; Owner: dub
--

CREATE SEQUENCE public."User_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public."User_id_seq" OWNER TO dub;

--
-- TOC entry 3744 (class 0 OID 0)
-- Dependencies: 256
-- Name: User_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: dub
--

ALTER SEQUENCE public."User_id_seq" OWNED BY public."User".id;


--
-- TOC entry 257 (class 1259 OID 16604)
-- Name: _ChannelToUser; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public."_ChannelToUser" (
    "A" integer NOT NULL,
    "B" integer NOT NULL
);


ALTER TABLE public."_ChannelToUser" OWNER TO dub;

--
-- TOC entry 258 (class 1259 OID 16607)
-- Name: _prisma_migrations; Type: TABLE; Schema: public; Owner: dub
--

CREATE TABLE public._prisma_migrations (
    id character varying(36) NOT NULL,
    checksum character varying(64) NOT NULL,
    finished_at timestamp with time zone,
    migration_name character varying(255) NOT NULL,
    logs text,
    rolled_back_at timestamp with time zone,
    started_at timestamp with time zone DEFAULT now() NOT NULL,
    applied_steps_count integer DEFAULT 0 NOT NULL
);


ALTER TABLE public._prisma_migrations OWNER TO dub;

--
-- TOC entry 3379 (class 2604 OID 16614)
-- Name: AuthorizedUserApplication id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."AuthorizedUserApplication" ALTER COLUMN id SET DEFAULT nextval('public."AuthorizedUserApplication_id_seq"'::regclass);


--
-- TOC entry 3380 (class 2604 OID 16615)
-- Name: Beatmap id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Beatmap" ALTER COLUMN id SET DEFAULT nextval('public."Beatmap_id_seq"'::regclass);


--
-- TOC entry 3383 (class 2604 OID 16616)
-- Name: Channel id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Channel" ALTER COLUMN id SET DEFAULT nextval('public."Channel_id_seq"'::regclass);


--
-- TOC entry 3385 (class 2604 OID 16617)
-- Name: GraphEntry id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."GraphEntry" ALTER COLUMN id SET DEFAULT nextval('public."GraphEntry_id_seq"'::regclass);


--
-- TOC entry 3387 (class 2604 OID 16618)
-- Name: Hwid id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Hwid" ALTER COLUMN id SET DEFAULT nextval('public."Hwid_id_seq"'::regclass);


--
-- TOC entry 3388 (class 2604 OID 16619)
-- Name: Hype id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Hype" ALTER COLUMN id SET DEFAULT nextval('public."Hype_id_seq"'::regclass);


--
-- TOC entry 3390 (class 2604 OID 16620)
-- Name: Integration id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Integration" ALTER COLUMN id SET DEFAULT nextval('public."Integration_id_seq"'::regclass);


--
-- TOC entry 3392 (class 2604 OID 16621)
-- Name: Message id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Message" ALTER COLUMN id SET DEFAULT nextval('public."Message_id_seq"'::regclass);


--
-- TOC entry 3394 (class 2604 OID 16622)
-- Name: MultiplayerAction id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerAction" ALTER COLUMN id SET DEFAULT nextval('public."MultiplayerAction_id_seq"'::regclass);


--
-- TOC entry 3397 (class 2604 OID 16623)
-- Name: MultiplayerMatch id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerMatch" ALTER COLUMN id SET DEFAULT nextval('public."MultiplayerMatch_id_seq"'::regclass);


--
-- TOC entry 3399 (class 2604 OID 16624)
-- Name: MultiplayerParticipant id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerParticipant" ALTER COLUMN id SET DEFAULT nextval('public."MultiplayerParticipant_id_seq"'::regclass);


--
-- TOC entry 3403 (class 2604 OID 16625)
-- Name: OAuthApplication id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."OAuthApplication" ALTER COLUMN id SET DEFAULT nextval('public."OAuthApplication_id_seq"'::regclass);


--
-- TOC entry 3405 (class 2604 OID 16626)
-- Name: PassKeys id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."PassKeys" ALTER COLUMN id SET DEFAULT nextval('public."PassKeys_id_seq"'::regclass);


--
-- TOC entry 3410 (class 2604 OID 16627)
-- Name: RelationShips id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."RelationShips" ALTER COLUMN id SET DEFAULT nextval('public."RelationShips_id_seq"'::regclass);


--
-- TOC entry 3411 (class 2604 OID 16628)
-- Name: Report id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Report" ALTER COLUMN id SET DEFAULT nextval('public."Report_id_seq"'::regclass);


--
-- TOC entry 3412 (class 2604 OID 16629)
-- Name: Score id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Score" ALTER COLUMN id SET DEFAULT nextval('public."Score_id_seq"'::regclass);


--
-- TOC entry 3415 (class 2604 OID 16630)
-- Name: User id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."User" ALTER COLUMN id SET DEFAULT nextval('public."User_id_seq"'::regclass);


--
-- TOC entry 3430 (class 2604 OID 16631)
-- Name: UserBadge id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."UserBadge" ALTER COLUMN id SET DEFAULT nextval('public."UserBadge_id_seq"'::regclass);


--
-- TOC entry 3482 (class 2604 OID 16862)
-- Name: UserGroup id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."UserGroup" ALTER COLUMN id SET DEFAULT nextval('public."UserGroup_id_seq"'::regclass);


--
-- TOC entry 3431 (class 2604 OID 16632)
-- Name: UserStats id; Type: DEFAULT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."UserStats" ALTER COLUMN id SET DEFAULT nextval('public."UserStats_id_seq"'::regclass);


--
-- TOC entry 3484 (class 2606 OID 16644)
-- Name: AuthorizedUserApplication AuthorizedUserApplication_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."AuthorizedUserApplication"
    ADD CONSTRAINT "AuthorizedUserApplication_pkey" PRIMARY KEY (id);


--
-- TOC entry 3487 (class 2606 OID 16646)
-- Name: Beatmap Beatmap_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Beatmap"
    ADD CONSTRAINT "Beatmap_pkey" PRIMARY KEY (id);


--
-- TOC entry 3490 (class 2606 OID 16648)
-- Name: Channel Channel_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Channel"
    ADD CONSTRAINT "Channel_pkey" PRIMARY KEY (id);


--
-- TOC entry 3492 (class 2606 OID 16650)
-- Name: GraphEntry GraphEntry_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."GraphEntry"
    ADD CONSTRAINT "GraphEntry_pkey" PRIMARY KEY (id);


--
-- TOC entry 3545 (class 2606 OID 16857)
-- Name: Group Group_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Group"
    ADD CONSTRAINT "Group_pkey" PRIMARY KEY (id);


--
-- TOC entry 3494 (class 2606 OID 16652)
-- Name: Hwid Hwid_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Hwid"
    ADD CONSTRAINT "Hwid_pkey" PRIMARY KEY (id);


--
-- TOC entry 3497 (class 2606 OID 16654)
-- Name: Hype Hype_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Hype"
    ADD CONSTRAINT "Hype_pkey" PRIMARY KEY (id);


--
-- TOC entry 3500 (class 2606 OID 16656)
-- Name: Integration Integration_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Integration"
    ADD CONSTRAINT "Integration_pkey" PRIMARY KEY (id);


--
-- TOC entry 3502 (class 2606 OID 16658)
-- Name: LinkedIntegration LinkedIntegration_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."LinkedIntegration"
    ADD CONSTRAINT "LinkedIntegration_pkey" PRIMARY KEY (id);


--
-- TOC entry 3504 (class 2606 OID 16660)
-- Name: Message Message_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Message"
    ADD CONSTRAINT "Message_pkey" PRIMARY KEY (id);


--
-- TOC entry 3506 (class 2606 OID 16662)
-- Name: MultiplayerAction MultiplayerAction_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerAction"
    ADD CONSTRAINT "MultiplayerAction_pkey" PRIMARY KEY (id);


--
-- TOC entry 3508 (class 2606 OID 16664)
-- Name: MultiplayerMatch MultiplayerMatch_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerMatch"
    ADD CONSTRAINT "MultiplayerMatch_pkey" PRIMARY KEY (id);


--
-- TOC entry 3510 (class 2606 OID 16666)
-- Name: MultiplayerParticipant MultiplayerParticipant_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerParticipant"
    ADD CONSTRAINT "MultiplayerParticipant_pkey" PRIMARY KEY (id);


--
-- TOC entry 3512 (class 2606 OID 16668)
-- Name: Notifications Notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Notifications"
    ADD CONSTRAINT "Notifications_pkey" PRIMARY KEY (id);


--
-- TOC entry 3514 (class 2606 OID 16670)
-- Name: OAuthApplication OAuthApplication_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."OAuthApplication"
    ADD CONSTRAINT "OAuthApplication_pkey" PRIMARY KEY (id);


--
-- TOC entry 3516 (class 2606 OID 16672)
-- Name: PassKeys PassKeys_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."PassKeys"
    ADD CONSTRAINT "PassKeys_pkey" PRIMARY KEY (id);


--
-- TOC entry 3519 (class 2606 OID 16674)
-- Name: PaymentEntry PaymentEntry_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."PaymentEntry"
    ADD CONSTRAINT "PaymentEntry_pkey" PRIMARY KEY (id);


--
-- TOC entry 3542 (class 2606 OID 16849)
-- Name: Post Post_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Post"
    ADD CONSTRAINT "Post_pkey" PRIMARY KEY (id);


--
-- TOC entry 3521 (class 2606 OID 16676)
-- Name: Punishment Punishment_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Punishment"
    ADD CONSTRAINT "Punishment_pkey" PRIMARY KEY (id);


--
-- TOC entry 3523 (class 2606 OID 16678)
-- Name: RelationShips RelationShips_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."RelationShips"
    ADD CONSTRAINT "RelationShips_pkey" PRIMARY KEY (id);


--
-- TOC entry 3525 (class 2606 OID 16680)
-- Name: Report Report_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Report"
    ADD CONSTRAINT "Report_pkey" PRIMARY KEY (id);


--
-- TOC entry 3527 (class 2606 OID 16682)
-- Name: Score Score_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Score"
    ADD CONSTRAINT "Score_pkey" PRIMARY KEY (id);


--
-- TOC entry 3533 (class 2606 OID 16684)
-- Name: UserBadge UserBadge_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."UserBadge"
    ADD CONSTRAINT "UserBadge_pkey" PRIMARY KEY (id);


--
-- TOC entry 3547 (class 2606 OID 16864)
-- Name: UserGroup UserGroup_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."UserGroup"
    ADD CONSTRAINT "UserGroup_pkey" PRIMARY KEY (id);


--
-- TOC entry 3535 (class 2606 OID 16686)
-- Name: UserStats UserStats_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."UserStats"
    ADD CONSTRAINT "UserStats_pkey" PRIMARY KEY (id);


--
-- TOC entry 3530 (class 2606 OID 16688)
-- Name: User User_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."User"
    ADD CONSTRAINT "User_pkey" PRIMARY KEY (id);


--
-- TOC entry 3540 (class 2606 OID 16690)
-- Name: _prisma_migrations _prisma_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public._prisma_migrations
    ADD CONSTRAINT _prisma_migrations_pkey PRIMARY KEY (id);


--
-- TOC entry 3485 (class 1259 OID 16691)
-- Name: Beatmap_checksum_key; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "Beatmap_checksum_key" ON public."Beatmap" USING btree (checksum);


--
-- TOC entry 3488 (class 1259 OID 16692)
-- Name: Channel_name_key; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "Channel_name_key" ON public."Channel" USING btree (name);


--
-- TOC entry 3543 (class 1259 OID 16865)
-- Name: Group_name_key; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "Group_name_key" ON public."Group" USING btree (name);


--
-- TOC entry 3495 (class 1259 OID 16693)
-- Name: Hwid_userId_key; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "Hwid_userId_key" ON public."Hwid" USING btree ("userId");


--
-- TOC entry 3498 (class 1259 OID 16694)
-- Name: Integration_name_key; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "Integration_name_key" ON public."Integration" USING btree (name);


--
-- TOC entry 3517 (class 1259 OID 16695)
-- Name: PassKeys_userIdHexed_key; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "PassKeys_userIdHexed_key" ON public."PassKeys" USING btree ("userIdHexed");


--
-- TOC entry 3536 (class 1259 OID 16696)
-- Name: UserStats_userId_key; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "UserStats_userId_key" ON public."UserStats" USING btree ("userId");


--
-- TOC entry 3528 (class 1259 OID 16697)
-- Name: User_email_key; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "User_email_key" ON public."User" USING btree (email);


--
-- TOC entry 3531 (class 1259 OID 16698)
-- Name: User_usernameSafe_key; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "User_usernameSafe_key" ON public."User" USING btree ("usernameSafe");


--
-- TOC entry 3537 (class 1259 OID 16699)
-- Name: _ChannelToUser_AB_unique; Type: INDEX; Schema: public; Owner: dub
--

CREATE UNIQUE INDEX "_ChannelToUser_AB_unique" ON public."_ChannelToUser" USING btree ("A", "B");


--
-- TOC entry 3538 (class 1259 OID 16700)
-- Name: _ChannelToUser_B_index; Type: INDEX; Schema: public; Owner: dub
--

CREATE INDEX "_ChannelToUser_B_index" ON public."_ChannelToUser" USING btree ("B");


--
-- TOC entry 3548 (class 2606 OID 16701)
-- Name: AuthorizedUserApplication AuthorizedUserApplication_oauthApplicationId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."AuthorizedUserApplication"
    ADD CONSTRAINT "AuthorizedUserApplication_oauthApplicationId_fkey" FOREIGN KEY ("oauthApplicationId") REFERENCES public."OAuthApplication"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3549 (class 2606 OID 16706)
-- Name: AuthorizedUserApplication AuthorizedUserApplication_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."AuthorizedUserApplication"
    ADD CONSTRAINT "AuthorizedUserApplication_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3550 (class 2606 OID 16711)
-- Name: GraphEntry GraphEntry_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."GraphEntry"
    ADD CONSTRAINT "GraphEntry_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3574 (class 2606 OID 16866)
-- Name: Group Group_badgeId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Group"
    ADD CONSTRAINT "Group_badgeId_fkey" FOREIGN KEY ("badgeId") REFERENCES public."UserBadge"(id) ON UPDATE CASCADE ON DELETE SET NULL;


--
-- TOC entry 3551 (class 2606 OID 16716)
-- Name: Hwid Hwid_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Hwid"
    ADD CONSTRAINT "Hwid_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3552 (class 2606 OID 16721)
-- Name: Hype Hype_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Hype"
    ADD CONSTRAINT "Hype_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3553 (class 2606 OID 16726)
-- Name: LinkedIntegration LinkedIntegration_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."LinkedIntegration"
    ADD CONSTRAINT "LinkedIntegration_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3554 (class 2606 OID 16731)
-- Name: Message Message_channelId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Message"
    ADD CONSTRAINT "Message_channelId_fkey" FOREIGN KEY ("channelId") REFERENCES public."Channel"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3555 (class 2606 OID 16736)
-- Name: Message Message_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Message"
    ADD CONSTRAINT "Message_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3556 (class 2606 OID 16741)
-- Name: MultiplayerAction MultiplayerAction_matchId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerAction"
    ADD CONSTRAINT "MultiplayerAction_matchId_fkey" FOREIGN KEY ("matchId") REFERENCES public."MultiplayerMatch"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3557 (class 2606 OID 16746)
-- Name: MultiplayerAction MultiplayerAction_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerAction"
    ADD CONSTRAINT "MultiplayerAction_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3558 (class 2606 OID 16751)
-- Name: MultiplayerParticipant MultiplayerParticipant_matchId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerParticipant"
    ADD CONSTRAINT "MultiplayerParticipant_matchId_fkey" FOREIGN KEY ("matchId") REFERENCES public."MultiplayerMatch"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3559 (class 2606 OID 16756)
-- Name: MultiplayerParticipant MultiplayerParticipant_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."MultiplayerParticipant"
    ADD CONSTRAINT "MultiplayerParticipant_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3560 (class 2606 OID 16761)
-- Name: Notifications Notifications_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Notifications"
    ADD CONSTRAINT "Notifications_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3561 (class 2606 OID 16766)
-- Name: OAuthApplication OAuthApplication_ownerId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."OAuthApplication"
    ADD CONSTRAINT "OAuthApplication_ownerId_fkey" FOREIGN KEY ("ownerId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3562 (class 2606 OID 16771)
-- Name: PassKeys PassKeys_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."PassKeys"
    ADD CONSTRAINT "PassKeys_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3563 (class 2606 OID 16776)
-- Name: PaymentEntry PaymentEntry_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."PaymentEntry"
    ADD CONSTRAINT "PaymentEntry_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3564 (class 2606 OID 16781)
-- Name: Punishment Punishment_appliedBy_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Punishment"
    ADD CONSTRAINT "Punishment_appliedBy_fkey" FOREIGN KEY ("appliedBy") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3565 (class 2606 OID 16786)
-- Name: Punishment Punishment_appliedTo_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Punishment"
    ADD CONSTRAINT "Punishment_appliedTo_fkey" FOREIGN KEY ("appliedTo") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3566 (class 2606 OID 16791)
-- Name: RelationShips RelationShips_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."RelationShips"
    ADD CONSTRAINT "RelationShips_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3567 (class 2606 OID 16796)
-- Name: Report Report_repporterId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Report"
    ADD CONSTRAINT "Report_repporterId_fkey" FOREIGN KEY ("repporterId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3568 (class 2606 OID 16801)
-- Name: Score Score_beatmapChecksum_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Score"
    ADD CONSTRAINT "Score_beatmapChecksum_fkey" FOREIGN KEY ("beatmapChecksum") REFERENCES public."Beatmap"(checksum) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3569 (class 2606 OID 16806)
-- Name: Score Score_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."Score"
    ADD CONSTRAINT "Score_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3570 (class 2606 OID 16811)
-- Name: UserBadge UserBadge_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."UserBadge"
    ADD CONSTRAINT "UserBadge_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3571 (class 2606 OID 16816)
-- Name: UserStats UserStats_userId_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."UserStats"
    ADD CONSTRAINT "UserStats_userId_fkey" FOREIGN KEY ("userId") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- TOC entry 3572 (class 2606 OID 16821)
-- Name: _ChannelToUser _ChannelToUser_A_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."_ChannelToUser"
    ADD CONSTRAINT "_ChannelToUser_A_fkey" FOREIGN KEY ("A") REFERENCES public."Channel"(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- TOC entry 3573 (class 2606 OID 16826)
-- Name: _ChannelToUser _ChannelToUser_B_fkey; Type: FK CONSTRAINT; Schema: public; Owner: dub
--

ALTER TABLE ONLY public."_ChannelToUser"
    ADD CONSTRAINT "_ChannelToUser_B_fkey" FOREIGN KEY ("B") REFERENCES public."User"(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- TOC entry 3724 (class 0 OID 0)
-- Dependencies: 5
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: dub
--

REVOKE USAGE ON SCHEMA public FROM PUBLIC;


-- Completed on 2024-06-08 19:18:55

--
-- PostgreSQL database dump complete
--

INSERT INTO public."User"
(id, username, "usernameSafe", "password", email, flags, permissions, country, "forgotPassword", "lastSeen", "createdAt", notes, "hypeRemaining", "donorUntil", "backgroundUrl", "ircPassword", "countryChangesRemaining", "oldUsernames", "usernameChangesRemaining", "lastHype", "userpageContent", coins)
VALUES(1, 'Mio', 'mio', '', 'ppy@ppy.sh', 14, 0, 'XX', false, '2023-07-26 16:33:00.368', '2022-12-03 20:08:43.000', NULL, 6, NULL, NULL, NULL, 0, NULL, 0, '2023-12-21 21:45:33.970', '', 0);
INSERT INTO public."UserStats"
(id, "userId", "rankedScoreStd", "rankedScoreTaiko", "rankedScoreCtb", "rankedScoreMania", "rankedScoreRx", "totalScoreStd", "totalScoreTaiko", "totalScoreCtb", "totalScoreMania", "totalScoreRx", "ppStd", "ppTaiko", "ppCtb", "ppMania", "ppRx", "avgAccStd", "avgAccTaiko", "avgAccCtb", "avgAccMania", "avgAccRx", "playCountStd", "playCountTaiko", "playCountCtb", "playCountMania", "playCountRx", "playTimeStd", "playTimeTaiko", "playTimeCtb", "playTimeMania", "playTimeRx", "maxComboStd", "maxComboTaiko", "maxComboCtb", "maxComboMania", "maxComboRx", "replaysWatchedStd", "replaysWatchedTaiko", "replaysWatchedCtb", "replaysWatchedMania", "replaysWatchedRx", "hitsStd", "hitsTaiko", "hitsCtb", "hitsMania", "hitsRx")
VALUES(1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 711, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);