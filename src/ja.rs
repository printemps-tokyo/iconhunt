//! Japanese keywords to English search words.
//!
//! Iconify search only matches English icon names and tags, so a query like
//! `家` finds nothing. This module maps common Japanese words for things icons
//! depict to the English words icon sets use. It is a fixed table, not a
//! translator: words it does not know are reported back to the user.
//!
//! Lookups are normalized first: full-width ASCII becomes ASCII, hiragana
//! becomes katakana, and ASCII is lowercased. Table keys are written in that
//! normalized form, so `ほーむ` and `ホーム` hit the same entry. Half-width
//! katakana is not folded.

/// One entry: Japanese spellings, then English search words (best first).
type Entry = (&'static [&'static str], &'static [&'static str]);

#[rustfmt::skip]
const TABLE: &[Entry] = &[
    // Navigation and layout
    (&["家", "イエ", "ホーム", "自宅"], &["home", "house"]),
    (&["メニュー", "ハンバーガー"], &["menu", "hamburger"]),
    (&["戻る", "モドル", "バック"], &["arrow-left", "back"]),
    (&["進む", "ススム", "次へ", "ツギヘ"], &["arrow-right", "next"]),
    (&["矢印", "ヤジルシ", "アロー"], &["arrow"]),
    (&["上", "ウエ"], &["arrow-up", "up"]),
    (&["下", "シタ"], &["arrow-down", "down"]),
    (&["左", "ヒダリ"], &["arrow-left", "left"]),
    (&["右", "ミギ"], &["arrow-right", "right"]),
    (&["閉じる", "トジル", "バツ", "クローズ"], &["close", "x"]),
    (&["チェック", "チェックマーク", "完了", "カンリョウ", "OK"], &["check"]),
    (&["プラス", "追加", "ツイカ", "足す"], &["plus", "add"]),
    (&["マイナス", "引く"], &["minus"]),
    (&["更新", "コウシン", "リロード", "再読み込み"], &["refresh", "reload"]),
    (&["展開", "テンカイ", "開く", "ヒラク"], &["expand", "open"]),
    (&["折りたたみ", "折り畳み", "オリタタミ"], &["collapse"]),
    (&["外部リンク"], &["external-link", "open-in-new"]),
    (&["リンク", "鎖"], &["link"]),
    (&["ダッシュボード"], &["dashboard"]),
    (&["グリッド", "格子"], &["grid"]),
    (&["リスト", "一覧", "イチラン"], &["list"]),
    (&["フィルター", "フィルタ", "絞り込み", "シボリコミ"], &["filter"]),
    (&["並べ替え", "ナラベカエ", "ソート"], &["sort"]),
    (&["全画面", "ゼンガメン", "フルスクリーン"], &["fullscreen"]),
    // Actions
    (&["検索", "ケンサク", "サーチ", "虫眼鏡", "虫めがね", "ムシメガネ"], &["search", "magnify"]),
    (&["設定", "セッテイ", "歯車", "ハグルマ", "ギア"], &["settings", "cog", "gear"]),
    (&["編集", "ヘンシュウ", "鉛筆", "エンピツ", "ペン"], &["edit", "pencil"]),
    (&["削除", "サクジョ", "ゴミ箱", "ゴミバコ", "ゴミ"], &["trash", "delete"]),
    (&["保存", "ホゾン", "セーブ"], &["save", "content-save"]),
    (&["コピー", "複製", "フクセイ"], &["copy", "content-copy"]),
    (&["貼り付け", "ハリツケ", "ペースト"], &["paste", "clipboard"]),
    (&["切り取り", "キリトリ", "ハサミ", "カット"], &["cut", "scissors"]),
    (&["ダウンロード"], &["download"]),
    (&["アップロード"], &["upload"]),
    (&["共有", "キョウユウ", "シェア"], &["share"]),
    (&["印刷", "インサツ", "プリンター", "プリンタ"], &["printer", "print"]),
    (&["送信", "ソウシン", "送る", "オクル"], &["send"]),
    (&["ログイン", "サインイン"], &["login", "log-in"]),
    (&["ログアウト", "サインアウト"], &["logout", "log-out"]),
    (&["元に戻す", "モトニモドス", "アンドゥ"], &["undo"]),
    (&["やり直し", "ヤリナオシ", "リドゥ"], &["redo"]),
    (&["拡大", "カクダイ", "ズームイン"], &["zoom-in"]),
    (&["縮小", "シュクショウ", "ズームアウト"], &["zoom-out"]),
    (&["再生", "サイセイ", "プレイ"], &["play"]),
    (&["一時停止", "イチジテイシ", "ポーズ"], &["pause"]),
    (&["停止", "テイシ", "ストップ"], &["stop"]),
    (&["録音", "ロクオン", "録画", "ロクガ"], &["record"]),
    (&["同期", "ドウキ", "シンク"], &["sync"]),
    (&["翻訳", "ホンヤク", "言語", "ゲンゴ"], &["translate", "language"]),
    // People and communication
    (&["人", "ヒト", "ユーザー", "ユーザ", "人物", "ジンブツ", "アカウント"], &["user", "account", "person"]),
    (&["人々", "グループ", "チーム", "メンバー"], &["users", "group", "account-group"]),
    (&["メール", "手紙", "テガミ", "封筒", "フウトウ"], &["mail", "email", "envelope"]),
    (&["電話", "デンワ"], &["phone"]),
    (&["スマホ", "スマートフォン", "携帯", "ケイタイ"], &["cellphone", "smartphone", "mobile"]),
    (&["チャット", "吹き出し", "フキダシ", "会話", "カイワ", "コメント"], &["chat", "message", "comment"]),
    (&["通知", "ツウチ", "ベル", "鈴", "スズ"], &["bell", "notification"]),
    (&["ハート", "イイネ", "好き", "スキ", "お気に入り", "オキニイリ"], &["heart", "favorite"]),
    (&["星", "ホシ", "スター"], &["star"]),
    (&["笑顔", "エガオ", "スマイル", "顔", "カオ"], &["smile", "face", "emoticon"]),
    (&["親指", "オヤユビ", "グッド"], &["thumb-up", "thumbs-up"]),
    (&["握手", "アクシュ"], &["handshake"]),
    (&["赤ちゃん", "アカチャン", "赤ん坊", "ベビー"], &["baby"]),
    (&["子供", "子ども", "コドモ", "キッズ"], &["child", "kid"]),
    // Objects
    (&["カレンダー", "日付", "ヒヅケ", "暦"], &["calendar"]),
    (&["時計", "トケイ", "時間", "ジカン", "時刻", "ジコク"], &["clock", "time"]),
    (&["目覚まし", "メザマシ", "アラーム"], &["alarm"]),
    (&["カメラ", "写真", "シャシン"], &["camera", "photo"]),
    (&["画像", "ガゾウ", "イメージ"], &["image", "picture"]),
    (&["動画", "ドウガ", "ビデオ", "映像", "エイゾウ"], &["video", "movie"]),
    (&["音楽", "オンガク", "ミュージック", "音符", "オンプ"], &["music", "note"]),
    (&["マイク"], &["microphone"]),
    (&["スピーカー", "音量", "オンリョウ", "音", "オト"], &["volume", "speaker", "sound"]),
    (&["ミュート", "消音", "ショウオン"], &["volume-off", "mute"]),
    (&["ヘッドホン", "ヘッドフォン"], &["headphones"]),
    (&["本", "ホン", "書籍", "ショセキ", "ブック"], &["book"]),
    (&["ノート", "メモ"], &["note", "notebook"]),
    (&["ファイル", "書類", "ショルイ", "ドキュメント", "文書", "ブンショ"], &["file", "document"]),
    (&["フォルダ", "フォルダー"], &["folder"]),
    (&["クリップ", "添付", "テンプ"], &["paperclip", "attachment"]),
    (&["鍵", "カギ", "キー"], &["key"]),
    (&["錠", "ロック", "施錠", "セジョウ"], &["lock"]),
    (&["解錠", "カイジョウ", "アンロック"], &["lock-open", "unlock"]),
    (&["盾", "タテ", "シールド", "セキュリティ", "安全", "アンゼン"], &["shield", "security"]),
    (&["目", "表示", "ヒョウジ", "見る", "ミル"], &["eye", "view"]),
    (&["非表示", "ヒヒョウジ"], &["eye-off", "hide"]),
    (&["電球", "デンキュウ", "アイデア", "アイディア", "ヒラメキ"], &["lightbulb", "bulb", "idea"]),
    (&["旗", "ハタ", "フラグ"], &["flag"]),
    (&["ブックマーク", "シオリ", "栞"], &["bookmark"]),
    (&["タグ", "札", "フダ", "ラベル"], &["tag", "label"]),
    (&["ピン", "地図ピン", "現在地", "ゲンザイチ", "位置", "イチ"], &["map-marker", "pin", "location"]),
    (&["地図", "チズ", "マップ"], &["map"]),
    (&["地球", "チキュウ", "世界", "セカイ", "グローブ", "ウェブ"], &["globe", "earth", "web"]),
    (&["方位", "ホウイ", "コンパス", "羅針盤"], &["compass"]),
    (&["ゴミ収集", "リサイクル"], &["recycle"]),
    (&["傘", "カサ"], &["umbrella"]),
    (&["ギフト", "プレゼント", "贈り物", "オクリモノ"], &["gift", "present"]),
    (&["トロフィー", "優勝", "ユウショウ"], &["trophy"]),
    (&["メダル"], &["medal"]),
    (&["王冠", "オウカン", "クラウン"], &["crown"]),
    (&["ロケット"], &["rocket"]),
    (&["爆弾", "バクダン"], &["bomb"]),
    (&["工具", "コウグ", "ツール", "レンチ", "スパナ"], &["wrench", "tools"]),
    (&["ハンマー", "金槌", "カナヅチ"], &["hammer"]),
    (&["電池", "デンチ", "バッテリー"], &["battery"]),
    (&["プラグ", "電源", "デンゲン", "コンセント"], &["power", "plug"]),
    (&["コード", "プログラム", "プログラミング"], &["code"]),
    (&["ターミナル", "端末", "タンマツ", "コンソール"], &["terminal", "console"]),
    (&["バグ", "虫", "ムシ"], &["bug"]),
    (&["データベース"], &["database"]),
    (&["サーバー", "サーバ"], &["server"]),
    (&["クラウド"], &["cloud"]),
    (&["パソコン", "コンピュータ", "コンピューター", "PC", "ノートパソコン"], &["laptop", "computer", "monitor"]),
    (&["モニター", "画面", "ガメン", "ディスプレイ"], &["monitor", "display"]),
    (&["キーボード"], &["keyboard"]),
    (&["マウス"], &["mouse"]),
    (&["ゲーム", "コントローラー"], &["gamepad", "controller"]),
    (&["テレビ", "TV"], &["television", "tv"]),
    (&["ラジオ"], &["radio"]),
    (&["電波", "デンパ", "ワイファイ", "WIFI", "無線", "ムセン"], &["wifi", "wireless"]),
    (&["ブルートゥース"], &["bluetooth"]),
    (&["QRコード", "QR"], &["qrcode", "qr-code"]),
    (&["バーコード"], &["barcode"]),
    (&["グラフ", "チャート", "統計", "トウケイ"], &["chart", "graph"]),
    (&["円グラフ", "エングラフ"], &["chart-pie", "pie-chart"]),
    (&["棒グラフ", "ボウグラフ"], &["chart-bar", "bar-chart"]),
    (&["表", "ヒョウ", "テーブル"], &["table"]),
    (&["電卓", "デンタク", "計算", "ケイサン", "計算機"], &["calculator"]),
    (&["ペンキ", "塗る", "ヌル", "パレット", "絵の具", "エノグ"], &["palette", "paint"]),
    (&["筆", "フデ", "ブラシ"], &["brush"]),
    // Commerce
    (&["カート", "買い物かご", "カイモノカゴ", "買い物", "カイモノ", "ショッピング"], &["cart", "shopping-cart"]),
    (&["バッグ", "鞄", "カバン", "袋", "フクロ"], &["bag", "shopping-bag"]),
    (&["店", "ミセ", "ショップ", "ストア", "店舗", "テンポ"], &["store", "shop", "storefront"]),
    (&["お金", "オカネ", "金", "カネ", "現金", "ゲンキン", "マネー"], &["money", "cash", "currency"]),
    (&["円", "エン"], &["currency-jpy", "yen"]),
    (&["ドル"], &["currency-usd", "dollar"]),
    (&["財布", "サイフ", "ウォレット"], &["wallet"]),
    (&["クレジットカード", "カード", "クレカ"], &["credit-card", "card"]),
    (&["レシート", "領収書", "リョウシュウショ", "請求書", "セイキュウショ"], &["receipt", "invoice"]),
    (&["割引", "ワリビキ", "セール", "パーセント"], &["percent", "discount", "sale"]),
    (&["配送", "ハイソウ", "トラック", "配達", "ハイタツ"], &["truck", "delivery"]),
    (&["箱", "ハコ", "パッケージ", "荷物", "ニモツ", "段ボール"], &["package", "box"]),
    (&["銀行", "ギンコウ"], &["bank"]),
    // Nature and weather
    (&["太陽", "タイヨウ", "晴れ", "ハレ", "日"], &["sun", "sunny"]),
    (&["月", "ツキ", "夜", "ヨル", "ダークモード"], &["moon"]),
    (&["雲", "クモ", "曇り", "クモリ"], &["cloud"]),
    (&["雨", "アメ"], &["rain", "weather-rainy"]),
    (&["雪", "ユキ"], &["snow", "snowflake"]),
    (&["雷", "カミナリ", "稲妻", "イナズマ", "電気", "デンキ"], &["lightning", "flash", "bolt"]),
    (&["風", "カゼ"], &["wind"]),
    (&["天気", "テンキ"], &["weather"]),
    (&["温度", "オンド", "温度計", "気温", "キオン"], &["thermometer", "temperature"]),
    (&["火", "炎", "ホノオ", "ファイア"], &["fire", "flame"]),
    (&["水", "ミズ", "シズク", "雫", "水滴"], &["water", "drop", "droplet"]),
    (&["花", "ハナ", "フラワー"], &["flower"]),
    (&["桜", "サクラ"], &["flower", "cherry-blossom"]),
    (&["木", "樹", "ツリー"], &["tree"]),
    (&["葉", "葉っぱ", "ハッパ", "リーフ", "エコ"], &["leaf", "eco"]),
    (&["山", "ヤマ"], &["mountain"]),
    (&["海", "ウミ", "波", "ナミ"], &["wave", "waves", "ocean"]),
    // Animals
    (&["猫", "ネコ", "キャット"], &["cat"]),
    (&["犬", "イヌ", "ドッグ"], &["dog"]),
    (&["鳥", "トリ"], &["bird"]),
    (&["魚", "サカナ"], &["fish"]),
    (&["ウサギ", "兎"], &["rabbit"]),
    (&["肉球", "ニクキュウ", "足跡", "アシアト", "ペット"], &["paw", "pet"]),
    // Places and transport
    (&["建物", "タテモノ", "ビル", "会社", "カイシャ", "オフィス"], &["building", "office"]),
    (&["学校", "ガッコウ"], &["school"]),
    (&["病院", "ビョウイン", "医療", "イリョウ"], &["hospital", "medical"]),
    (&["車", "クルマ", "自動車", "ジドウシャ"], &["car"]),
    (&["自転車", "ジテンシャ"], &["bicycle", "bike"]),
    (&["電車", "デンシャ", "鉄道", "テツドウ", "列車", "レッシャ"], &["train"]),
    (&["バス"], &["bus"]),
    (&["飛行機", "ヒコウキ"], &["airplane", "plane"]),
    (&["船", "フネ"], &["ship", "boat"]),
    (&["歩く", "アルク", "徒歩", "トホ"], &["walk"]),
    // Food
    (&["食事", "ショクジ", "レストラン", "食べ物", "タベモノ", "ナイフとフォーク"], &["food", "restaurant", "silverware"]),
    (&["コーヒー", "カフェ"], &["coffee", "cup"]),
    (&["ビール", "お酒", "オサケ", "酒"], &["beer", "glass"]),
    (&["ケーキ", "誕生日", "タンジョウビ"], &["cake", "birthday"]),
    (&["リンゴ", "果物", "クダモノ"], &["apple", "fruit"]),
    // Status
    (&["情報", "ジョウホウ", "インフォ"], &["information", "info"]),
    (&["警告", "ケイコク", "注意", "チュウイ", "危険", "キケン"], &["alert", "warning"]),
    (&["エラー", "禁止", "キンシ"], &["error", "cancel", "block"]),
    (&["ヘルプ", "ハテナ", "質問", "シツモン", "疑問"], &["help", "question"]),
    (&["読み込み", "ヨミコミ", "ローディング", "スピナー"], &["loading", "spinner"]),
    (&["ハッシュタグ"], &["pound", "hashtag"]),
    (&["アットマーク"], &["at"]),
    (&["健康", "ケンコウ", "医者", "イシャ"], &["health", "doctor"]),
    (&["薬", "クスリ"], &["pill", "medicine"]),
    (&["注射", "チュウシャ", "ワクチン"], &["syringe", "needle"]),
    (&["マスク"], &["face-mask", "mask"]),
    (&["心臓", "シンゾウ", "心拍", "シンパク"], &["heart-pulse", "heart"]),
    (&["脳", "頭脳", "ズノウ"], &["brain"]),
    (&["地震", "ジシン"], &["earthquake"]),
    // Body and clothing
    (&["手"], &["hand"]),
    (&["足"], &["foot", "footprint"]),
    (&["眼鏡", "メガネ"], &["glasses"]),
    (&["帽子", "ボウシ"], &["hat"]),
    (&["服", "フク", "シャツ", "Tシャツ"], &["shirt", "tshirt"]),
    (&["靴", "クツ", "シューズ"], &["shoe"]),
    // Devices and security
    (&["充電", "ジュウデン"], &["battery-charging"]),
    (&["指紋", "シモン"], &["fingerprint"]),
    (&["顔認証", "カオニンショウ"], &["face-recognition", "face-id"]),
    (&["数字", "スウジ", "番号", "バンゴウ"], &["numeric", "number"]),
    (&["パズル", "ジグソー"], &["puzzle"]),
    // Places, transport and money
    (&["テント"], &["tent"]),
    (&["工場", "コウジョウ"], &["factory"]),
    (&["ホテル", "宿泊", "シュクハク"], &["hotel", "bed"]),
    (&["駅", "エキ"], &["train-station", "train"]),
    (&["信号", "シンゴウ", "信号機"], &["traffic-light"]),
    (&["ガソリン", "給油", "キュウユ", "ガソリンスタンド"], &["gas-station", "fuel"]),
    (&["ユーロ"], &["euro"]),
    (&["ビットコイン", "仮想通貨", "カソウツウカ", "暗号資産"], &["bitcoin"]),
    (&["株", "株価", "カブカ", "株式", "カブシキ"], &["stock", "chart-line"]),
    (&["上昇", "ジョウショウ", "値上がり"], &["trending-up"]),
    (&["下落", "ゲラク", "下降", "カコウ", "値下がり"], &["trending-down"]),
];

/// The result of looking up a query.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Translation {
    /// English queries to send, best first. Empty when nothing was understood.
    pub queries: Vec<String>,
    /// Japanese parts of the query that were mapped, with their keywords.
    pub mapped: Vec<(String, Vec<String>)>,
    /// Parts of the query the table does not know.
    pub unknown: Vec<String>,
}

impl Translation {
    pub fn translated(&self) -> bool {
        !self.mapped.is_empty()
    }
}

/// Most English queries one Japanese query expands to.
pub const MAX_QUERIES: usize = 3;

/// Folds full-width ASCII to ASCII and hiragana to katakana, and lowercases.
pub fn normalize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\u{3000}' => ' ',
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFF01 + 0x21).unwrap_or(c),
            '\u{3041}'..='\u{3096}' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
            _ => c,
        })
        .flat_map(char::to_lowercase)
        .collect()
}

fn lookup(word: &str) -> Option<&'static [&'static str]> {
    TABLE
        .iter()
        .find(|(keys, _)| keys.iter().any(|k| normalize(k) == word))
        .map(|(_, en)| *en)
}

/// Splits a Japanese token into dictionary words by longest match. Characters
/// that start no known word are collected as unknown runs.
/// Particles between words (`猫の写真`) that carry no keyword of their own.
const PARTICLES: &[&str] = &["ノ", "ト", "ヲ", "ニ", "ガ", "ハ", "ヘ", "デ", "モ", "ヤ"];

fn is_kana(c: char) -> bool {
    matches!(c, '\u{30A0}'..='\u{30FF}')
}

fn segment(token: &str) -> (Vec<(String, &'static [&'static str])>, Vec<String>) {
    let chars: Vec<char> = token.chars().collect();
    let mut words = Vec::new();
    let mut unknown = Vec::new();
    let mut pending = String::new();
    let mut i = 0;
    while i < chars.len() {
        let mut hit = None;
        for end in (i + 1..=chars.len()).rev() {
            // A lone kana inside a longer word is almost always part of a
            // word the table lacks (the キ of キャンプ is not a tree).
            if end - i == 1 && is_kana(chars[i]) {
                continue;
            }
            let candidate: String = chars[i..end].iter().collect();
            if let Some(en) = lookup(&candidate) {
                hit = Some((candidate, en, end));
                break;
            }
        }
        match hit {
            Some((word, en, end)) => {
                flush(&mut pending, &mut unknown);
                words.push((word, en));
                i = end;
            }
            None => {
                pending.push(chars[i]);
                i += 1;
            }
        }
    }
    flush(&mut pending, &mut unknown);
    (words, unknown)
}

fn flush(pending: &mut String, unknown: &mut Vec<String>) {
    let run = std::mem::take(pending);
    if !run.is_empty() && !PARTICLES.contains(&run.as_str()) {
        unknown.push(run);
    }
}

/// Turns a user query into English search queries.
///
/// ASCII words pass through unchanged. A query with no Japanese at all comes
/// back as itself, so English searches behave exactly as on the API.
pub fn translate(query: &str) -> Translation {
    let normalized = normalize(query);
    let mut out = Translation::default();
    // Each slot holds the alternatives for one word of the query.
    let mut slots: Vec<Vec<String>> = Vec::new();
    for token in normalized.split(|c: char| c.is_whitespace() || matches!(c, '、' | ',' | '・')) {
        if token.is_empty() {
            continue;
        }
        if token.is_ascii() {
            slots.push(vec![token.to_string()]);
            continue;
        }
        if let Some(en) = lookup(token) {
            out.mapped.push((
                token.to_string(),
                en.iter().map(|s| s.to_string()).collect(),
            ));
            slots.push(en.iter().map(|s| s.to_string()).collect());
            continue;
        }
        let (words, unknown) = segment(token);
        for (word, en) in words {
            out.mapped
                .push((word, en.iter().map(|s| s.to_string()).collect()));
            slots.push(en.iter().map(|s| s.to_string()).collect());
        }
        out.unknown.extend(unknown);
    }
    if slots.is_empty() {
        return out;
    }
    // The first query uses every word's best keyword; later ones swap in the
    // alternatives of one word at a time.
    let best: Vec<String> = slots.iter().map(|s| s[0].clone()).collect();
    out.queries.push(best.join(" "));
    'outer: for (i, slot) in slots.iter().enumerate() {
        for alt in slot.iter().skip(1) {
            if out.queries.len() >= MAX_QUERIES {
                break 'outer;
            }
            let mut words = best.clone();
            words[i] = alt.clone();
            let q = words.join(" ");
            if !out.queries.contains(&q) {
                out.queries.push(q);
            }
        }
    }
    out
}

/// Number of Japanese spellings in the table, for `--help` style summaries.
pub fn entry_count() -> usize {
    TABLE.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_width_and_kana() {
        assert_eq!(normalize("ほーむ"), "ホーム");
        assert_eq!(normalize("ＱＲコード"), "qrコード");
        assert_eq!(normalize("家\u{3000}Home"), "家 home");
    }

    #[test]
    fn english_passes_through() {
        let t = translate("Arrow Left");
        assert_eq!(t.queries, vec!["arrow left"]);
        assert!(!t.translated());
        assert!(t.unknown.is_empty());
    }

    #[test]
    fn maps_single_words_with_alternatives() {
        let t = translate("家");
        assert_eq!(t.queries, vec!["home", "house"]);
        assert_eq!(
            t.mapped,
            vec![("家".to_string(), vec!["home".into(), "house".into()])]
        );
        assert_eq!(translate("ごみばこ").queries[0], "trash");
        assert_eq!(translate("設定").queries, vec!["settings", "cog", "gear"]);
        assert_eq!(translate("しもん").queries, vec!["fingerprint"]);
        assert_eq!(translate("株価").queries[0], "stock");
    }

    #[test]
    fn segments_compound_words_and_reports_unknown() {
        let t = translate("猫の写真");
        assert_eq!(t.queries[0], "cat camera");
        assert!(t.unknown.is_empty(), "{:?}", t.unknown);
        let t = translate("キャンプ");
        assert!(t.queries.is_empty());
        assert_eq!(t.unknown, vec!["キャンプ"]);
        let t = translate("ぴよぴよ");
        assert!(t.queries.is_empty());
        assert_eq!(t.unknown, vec!["ピヨピヨ"]);
    }

    #[test]
    fn mixes_english_and_japanese() {
        let t = translate("矢印 circle");
        assert_eq!(t.queries, vec!["arrow circle"]);
    }

    #[test]
    fn caps_the_number_of_queries() {
        assert!(translate("人 メール").queries.len() <= MAX_QUERIES);
    }

    #[test]
    fn table_keys_are_unique_after_normalizing() {
        let mut seen = std::collections::BTreeMap::new();
        for (i, (keys, en)) in TABLE.iter().enumerate() {
            assert!(!en.is_empty());
            for k in *keys {
                let n = normalize(k);
                if let Some(prev) = seen.insert(n.clone(), i) {
                    panic!("'{k}' ({n}) appears in entries {prev} and {i}");
                }
            }
        }
    }
}
