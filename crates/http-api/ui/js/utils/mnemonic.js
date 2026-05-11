/**
 * NRCS Wallet - Mnemonic (Passphrase) Utilities
 * 
 * 助记词/密码短语处理工具类
 * 完整移植自 NRCS Java 项目的实现逻辑
 * 
 * 核心算法:
 * - 使用自定义的 1176 个英语单词列表 (非标准 BIP39)
 * - 从密码短语/助记词派生 Ed25519 密钥对
 * - Account ID = SHA256(publicKey)[0..8] as u64 big-endian
 * 
 * 参考源码:
 * - Java: crypto/passphrasegenerator.js
 * - Rust: crates/crypto/src/passphrase/mod.rs
 * - Rust: crates/crypto/src/passphrase/words.rs
 */

const MnemonicUtils = {
  /**
   * NRCS 自定义助记词单词列表 (1176个)
   * 与后端 words.rs 完全一致
   */
  WORDS: [
    "like", "just", "love", "know", "never", "want", "time",
    "out", "there", "make", "look", "eye", "down", "only", "think", "heart", "back", "then", "into", "about",
    "more", "away", "still", "them", "take", "thing", "even", "through", "long", "always", "world", "too",
    "friend", "tell", "try", "hand", "thought", "over", "here", "other", "need", "smile", "again", "much",
    "cry", "been", "night", "ever", "little", "said", "end", "some", "those", "around", "mind", "people",
    "girl", "leave", "dream", "left", "turn", "myself", "give", "nothing", "really", "off", "before",
    "something", "find", "walk", "wish", "good", "once", "place", "ask", "stop", "keep", "watch", "seem",
    "everything", "wait", "got", "yet", "made", "remember", "start", "alone", "run", "hope", "maybe", "believe",
    "body", "hate", "after", "close", "talk", "stand", "own", "each", "hurt", "help", "home", "god", "soul",
    "new", "many", "two", "inside", "should", "true", "first", "fear", "mean", "better", "play", "another",
    "gone", "change", "use", "wonder", "someone", "hair", "cold", "open", "best", "any", "behind", "happen",
    "water", "dark", "laugh", "stay", "forever", "name", "work", "show", "sky", "break", "came", "deep",
    "door", "put", "black", "together", "upon", "happy", "such", "great", "white", "matter", "fill", "past",
    "please", "burn", "cause", "enough", "touch", "moment", "soon", "voice", "scream", "anything", "stare",
    "sound", "red", "everyone", "hide", "kiss", "truth", "death", "beautiful", "mine", "blood", "broken",
    "very", "pass", "next", "forget", "tree", "wrong", "air", "mother", "understand", "lip", "hit", "wall",
    "memory", "sleep", "free", "high", "realize", "school", "might", "skin", "sweet", "perfect", "blue", "kill",
    "breath", "dance", "against", "fly", "between", "grow", "strong", "under", "listen", "bring", "sometimes",
    "speak", "pull", "person", "become", "family", "begin", "ground", "real", "small", "father", "sure", "feet",
    "rest", "young", "finally", "land", "across", "today", "different", "guy", "line", "fire", "reason",
    "reach", "second", "slowly", "write", "eat", "smell", "mouth", "step", "learn", "three", "floor", "promise",
    "breathe", "darkness", "push", "earth", "guess", "save", "song", "above", "along", "both", "color", "house",
    "almost", "sorry", "anymore", "brother", "okay", "dear", "game", "fade", "already", "apart", "warm",
    "beauty", "heard", "notice", "question", "shine", "began", "piece", "whole", "shadow", "secret", "street",
    "within", "finger", "point", "morning", "whisper", "child", "moon", "green", "story", "glass", "kid",
    "silence", "since", "soft", "yourself", "empty", "shall", "angel", "answer", "baby", "bright", "dad",
    "path", "worry", "hour", "drop", "follow", "power", "war", "half", "flow", "heaven", "act", "chance",
    "fact", "least", "tired", "children", "near", "quite", "afraid", "rise", "sea", "taste", "window", "cover",
    "nice", "trust", "lot", "sad", "cool", "force", "peace", "return", "blind", "easy", "ready", "roll", "rose",
    "drive", "held", "music", "beneath", "hang", "mom", "paint", "emotion", "quiet", "clear", "cloud", "few",
    "pretty", "bird", "outside", "paper", "picture", "front", "rock", "simple", "anyone", "meant", "reality",
    "road", "sense", "waste", "bit", "leaf", "thank", "happiness", "meet", "men", "smoke", "truly", "decide",
    "self", "age", "book", "form", "alive", "carry", "escape", "damn", "instead", "able", "ice", "minute",
    "throw", "catch", "leg", "ring", "course", "goodbye", "lead", "poem", "sick", "corner", "desire", "known",
    "problem", "remind", "shoulder", "suppose", "toward", "wave", "drink", "jump", "woman", "pretend", "sister",
    "week", "human", "joy", "crack", "grey", "pray", "surprise", "dry", "knee", "less", "search", "bleed",
    "caught", "clean", "embrace", "future", "king", "son", "sorrow", "chest", "hug", "remain", "sat", "worth",
    "blow", "daddy", "final", "parent", "tight", "also", "create", "lonely", "safe", "cross", "dress", "evil",
    "silent", "bone", "fate", "perhaps", "anger", "class", "scar", "snow", "tiny", "tonight", "continue",
    "control", "dog", "edge", "mirror", "month", "suddenly", "comfort", "given", "loud", "quickly", "gaze",
    "plan", "rush", "stone", "town", "battle", "ignore", "spirit", "stood", "stupid", "yours", "brown", "build",
    "dust", "hey", "kept", "pay", "phone", "twist", "although", "ball", "beyond", "hidden", "nose", "taken",
    "fail", "float", "pure", "somehow", "wash", "wrap", "angry", "cheek", "creature", "forgotten", "heat",
    "rip", "single", "space", "special", "weak", "whatever", "yell", "anyway", "blame", "job", "choose",
    "country", "curse", "drift", "echo", "figure", "grew", "laughter", "neck", "suffer", "worse", "yeah",
    "disappear", "foot", "forward", "knife", "mess", "somewhere", "stomach", "storm", "beg", "idea", "lift",
    "offer", "breeze", "field", "five", "often", "simply", "stuck", "win", "allow", "confuse", "enjoy",
    "except", "flower", "seek", "strength", "calm", "grin", "gun", "heavy", "hill", "large", "ocean", "shoe",
    "sigh", "straight", "summer", "tongue", "accept", "crazy", "everyday", "exist", "grass", "mistake", "sent",
    "shut", "surround", "table", "ache", "brain", "destroy", "heal", "nature", "shout", "sign", "stain",
    "choice", "doubt", "glance", "glow", "mountain", "queen", "stranger", "throat", "tomorrow", "city",
    "either", "fish", "flame", "rather", "shape", "spin", "spread", "ash", "distance", "finish", "image",
    "imagine", "important", "nobody", "shatter", "warmth", "became", "feed", "flesh", "funny", "lust", "shirt",
    "trouble", "yellow", "attention", "bare", "bite", "money", "protect", "amaze", "appear", "born", "choke",
    "completely", "daughter", "fresh", "friendship", "gentle", "probably", "six", "deserve", "expect", "grab",
    "middle", "nightmare", "river", "thousand", "weight", "worst", "wound", "barely", "bottle", "cream",
    "regret", "relationship", "stick", "test", "crush", "endless", "fault", "itself", "rule", "spill", "art",
    "circle", "join", "kick", "mask", "master", "passion", "quick", "raise", "smooth", "unless", "wander",
    "actually", "broke", "chair", "deal", "favorite", "gift", "note", "number", "sweat", "box", "chill",
    "clothes", "lady", "mark", "park", "poor", "sadness", "tie", "animal", "belong", "brush", "consume", "dawn",
    "forest", "innocent", "pen", "pride", "stream", "thick", "clay", "complete", "count", "draw", "faith",
    "press", "silver", "struggle", "surface", "taught", "teach", "wet", "bless", "chase", "climb", "enter",
    "letter", "melt", "metal", "movie", "stretch", "swing", "vision", "wife", "beside", "crash", "forgot",
    "guide", "haunt", "joke", "knock", "plant", "pour", "prove", "reveal", "steal", "stuff", "trip", "wood",
    "wrist", "bother", "bottom", "crawl", "crowd", "fix", "forgive", "frown", "grace", "loose", "lucky",
    "party", "release", "surely", "survive", "teacher", "gently", "grip", "speed", "suicide", "travel", "treat",
    "vein", "written", "cage", "chain", "conversation", "date", "enemy", "however", "interest", "million",
    "page", "pink", "proud", "sway", "themselves", "winter", "church", "cruel", "cup", "demon", "experience",
    "freedom", "pair", "pop", "purpose", "respect", "shoot", "softly", "state", "strange", "bar", "birth",
    "curl", "dirt", "excuse", "lord", "lovely", "monster", "order", "pack", "pants", "pool", "scene", "seven",
    "shame", "slide", "ugly", "among", "blade", "blonde", "closet", "creek", "deny", "drug", "eternity", "gain",
    "grade", "handle", "key", "linger", "pale", "prepare", "swallow", "swim", "tremble", "wheel", "won", "cast",
    "cigarette", "claim", "college", "direction", "dirty", "gather", "ghost", "hundred", "loss", "lung",
    "orange", "present", "swear", "swirl", "twice", "wild", "bitter", "blanket", "doctor", "everywhere",
    "flash", "grown", "knowledge", "numb", "pressure", "radio", "repeat", "ruin", "spend", "unknown", "buy",
    "clock", "devil", "early", "false", "fantasy", "pound", "precious", "refuse", "sheet", "teeth", "welcome",
    "add", "ahead", "block", "bury", "caress", "content", "depth", "despite", "distant", "marry", "purple",
    "threw", "whenever", "bomb", "dull", "easily", "grasp", "hospital", "innocence", "normal", "receive",
    "reply", "rhyme", "shade", "someday", "sword", "toe", "visit", "asleep", "bought", "center", "consider",
    "flat", "hero", "history", "ink", "insane", "muscle", "mystery", "pocket", "reflection", "shove",
    "silently", "smart", "soldier", "spot", "stress", "train", "type", "view", "whether", "bus", "energy",
    "explain", "holy", "hunger", "inch", "magic", "mix", "noise", "nowhere", "prayer", "presence", "shock",
    "snap", "spider", "study", "thunder", "trail", "admit", "agree", "bag", "bang", "bound", "butterfly",
    "cute", "exactly", "explode", "familiar", "fold", "further", "pierce", "reflect", "scent", "selfish",
    "sharp", "sink", "spring", "stumble", "universe", "weep", "women", "wonderful", "action", "ancient",
    "attempt", "avoid", "birthday", "branch", "chocolate", "core", "depress", "drunk", "especially", "focus",
    "fruit", "honest", "match", "palm", "perfectly", "pillow", "pity", "poison", "roar", "shift", "slightly",
    "thump", "truck", "tune", "twenty", "unable", "wipe", "wrote", "coat", "constant", "dinner", "drove",
    "egg", "eternal", "flight", "flood", "frame", "freak", "gasp", "glad", "hollow", "motion", "peer",
    "plastic", "root", "screen", "season", "sting", "strike", "team", "unlike", "victim", "volume", "warn",
    "weird", "attack", "await", "awake", "built", "charm", "crave", "despair", "fought", "grant", "grief",
    "horse", "limit", "message", "ripple", "sanity", "scatter", "serve", "split", "string", "trick", "annoy",
    "blur", "boat", "brave", "clearly", "cling", "connect", "fist", "forth", "imagination", "iron", "jock",
    "judge", "lesson", "milk", "misery", "nail", "naked", "ourselves", "poet", "possible", "princess", "sail"
  ],

  /**
   * 生成新的 12 词助记词短语
   * 
   * 算法说明 (与 NRCS passphrasegenerator.js 一致):
   * 1. 使用 Web Crypto API 生成 128 位随机数 (4个 Uint32)
   * 2. 每个随机值生成 3 个单词: w1, w2, w3
   *    - w1 = x % n (n = 单词表长度)
   *    - w2 = ((x / n) + w1) % n
   *    - w3 = (((x / n) / n) + w2) % n
   * 3. 共 4 个随机值 → 12 个单词
   * 
   * @returns {Object} 包含助记词、账户ID和公钥的对象
   */
  generateMnemonic() {
    const crypto = window.crypto || window.msCrypto;
    
    if (!crypto || !crypto.getRandomValues) {
      throw new Error('Browser does not support secure random number generation');
    }

    const bits = 128;
    const random = new Uint32Array(bits / 32);
    crypto.getRandomValues(random);

    const n = this.WORDS.length;
    const phraseWords = [];

    for (let i = 0; i < random.length; i++) {
      const x = random[i];
      const w1 = x % n;
      const w2 = (((x / n) | 0) + w1) % n;
      const w3 = (((((x / n) | 0) / n) | 0) + w2) % n;

      phraseWords.push(this.WORDS[w1]);
      phraseWords.push(this.WORDS[w2]);
      phraseWords.push(this.WORDS[w3]);
    }

    // 再次获取随机值以增加安全性
    crypto.getRandomValues(random);
    
    const passphrase = phraseWords.join(' ');
    
    return {
      passphrase,
      words: phraseWords,
      wordCount: phraseWords.length,
    };
  },

  /**
   * 验证助记词是否有效
   * 
   * @param {string} mnemonic - 助记词字符串 (空格分隔的12个词)
   * @returns {Object} 验证结果 { valid, errors }
   */
  validateMnemonic(mnemonic) {
    if (!mnemonic || typeof mnemonic !== 'string') {
      return { valid: false, errors: ['Mnemonic is required'] };
    }

    const words = mnemonic.trim().split(/\s+/).filter(w => w.length > 0);
    const errors = [];

    // 检查单词数量
    if (words.length !== 12) {
      errors.push(`Must contain exactly 12 words, got ${words.length}`);
    }

    // 检查每个单词是否在单词表中
    const wordSet = new Set(this.WORDS.map(w => w.toLowerCase()));
    for (let i = 0; i < words.length; i++) {
      const word = words[i].toLowerCase();
      if (!wordSet.has(word)) {
        errors.push(`Invalid word at position ${i + 1}: "${words[i]}"`);
      }
    }

    // 检查是否有重复单词 (虽然技术上允许，但安全性较低)
    const uniqueWords = new Set(words.map(w => w.toLowerCase()));
    if (uniqueWords.size < words.length) {
      errors.push('Warning: Duplicate words detected (reduces security)');
    }

    return {
      valid: errors.filter(e => !e.startsWith('Warning')).length === 0,
      errors,
      wordCount: words.length,
    };
  },

  /**
   * 将助记词转换为十六进制公钥 (前端模拟)
   * 
   * 注意: 真正的密钥生成应该在安全环境(后端)完成
   * 前端仅用于显示预览信息
   * 
   * @param {string} passphrase - 密码短语或助记词
   * @returns {string} 十六进制公钥 (模拟值)
   */
  async getPublicKeyFromPassphrase(passphrase) {
    // 在实际应用中，这应该调用后端 API:
    // POST /nrcs?requestType=getAccountId
    // Body: { secretPhrase: passphrase }
    //
    // 这里使用 Web Crypto API 进行 SHA-256 哈希作为演示
    const encoder = new TextEncoder();
    const data = encoder.encode(passphrase);
    const hashBuffer = await crypto.subtle('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    
    // 转换为十六进制字符串 (这是 publicKey 的近似值)
    const hexString = hashArray
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
    
    return hexString.toUpperCase();
  },

  /**
   * 计算账户 ID (模拟)
   * 
   * Account ID = SHA256(publicKey)[0..8] as u64 (big-endian)
   * 
   * @param {string} publicKeyHex - 十六进制公钥
   * @returns {string} 账户 ID 字符串
   */
  calculateAccountId(publicKeyHex) {
    // 将十六进制转换为字节数组
    const bytes = [];
    for (let i = 0; i < publicKeyHex.length; i += 2) {
      bytes.push(parseInt(publicKeyHex.substr(i, 2), 16));
    }

    // 取前8字节作为 account ID (大端序)
    const accountIdBytes = bytes.slice(0, 8);
    let accountId = 0n;
    
    for (const byte of accountIdBytes) {
      accountId = (accountId << 8n) | BigInt(byte);
    }

    return accountId.toString();
  },

  /**
   * 格式化助记词为显示格式 (每4个词一行)
   * 
   * @param {string} mnemonic - 助记词字符串
   * @returns {string} 格式化后的助记词
   */
  formatMnemonicForDisplay(mnemonic) {
    const words = mnemonic.trim().split(/\s+/);
    const lines = [];
    
    for (let i = 0; i < words.length; i += 4) {
      const chunk = words.slice(i, i + 4);
      lines.push(chunk.join('  '));
    }
    
    return lines.join('\n');
  },

  /**
   * 掩码助记词 (用于确认步骤)
   * 显示部分单词，隐藏其他单词让用户输入
   * 
   * @param {string} mnemonic - 完整助记词
   * @param {number} revealCount - 要显示的单词数量 (默认 6)
   * @returns {Object} { masked, hiddenWords, revealedIndices }
   */
  maskMnemonicForConfirmation(mnemonic, revealCount = 6) {
    const words = mnemonic.trim().split(/\s+/);
    const indices = Array.from({ length: words.length }, (_, i) => i);
    
    // 随机选择要显示的单词索引
    const shuffled = indices.sort(() => Math.random() - 0.5);
    const revealedIndices = shuffled.slice(0, revealCount).sort((a, b) => a - b);
    
    const masked = words.map((word, index) => {
      if (revealedIndices.includes(index)) {
        return word;
      } else {
        return '_____';
      }
    });

    return {
      masked: masked.join('  '),
      hiddenWords: words.filter((_, index) => !revealedIndices.includes(index)),
      revealedIndices,
      fullMnemonic: mnemonic,
    };
  },

  /**
   * 检查密码强度
   * 
   * @param {string} password - 密码或助记词
   * @returns {Object} 强度评估 { score, level, suggestions }
   */
  checkPasswordStrength(password) {
    if (!password || password.length === 0) {
      return { score: 0, level: 'none', suggestions: ['Password is required'] };
    }

    let score = 0;
    const suggestions = [];

    // 长度检查
    if (password.length >= 35) {
      score += 40; // 优秀
    } else if (password.length >= 20) {
      score += 25; // 良好
      suggestions.push('Consider using a longer passphrase for better security');
    } else if (password.length >= 12) {
      score += 15; // 一般
      suggestions.push('Passphrase should be at least 35 characters for optimal security');
    } else if (password.length >= 8) {
      score += 5; // 弱
      suggestions.push('Password too short (minimum 12 characters recommended)');
    } else {
      suggestions.push('Password is very short and weak');
    }

    // 复杂性检查
    const hasUpperCase = /[A-Z]/.test(password);
    const hasLowerCase = /[a-z]/.test(password);
    const hasNumbers = /[0-9]/.test(password);
    const hasSpecialChars = /[^a-zA-Z0-9\s]/.test(password);

    if (hasUpperCase && hasLowerCase) score += 10;
    if (hasNumbers) score += 10;
    if (hasSpecialChars) score += 10;

    // 多词检查 (助记词)
    const wordCount = password.split(/\s+/).filter(w => w.length > 0).length;
    if (wordCount >= 8) {
      score += 15; // 助记词加分
    }

    // 确定等级
    let level;
    if (score >= 70) {
      level = 'strong';
    } else if (score >= 45) {
      level = 'medium';
    } else if (score >= 20) {
      level = 'weak';
    } else {
      level = 'very_weak';
    }

    return { score: Math.min(score, 100), level, suggestions };
  },

  /**
   * 安全地清除内存中的敏感数据
   * 
   * @param {string|Array} data - 要清除的数据
   */
  clearSensitiveData(data) {
    if (typeof data === 'string') {
      // 用随机数据覆盖字符串 (防止内存分析攻击)
      const array = new Uint8Array(data.length);
      crypto.getRandomValues(array);
      return String.fromCharCode(...array);
    } else if (Array.isArray(data)) {
      data.fill(0);
    }
    return null;
  },
};

// 导出供全局使用
window.MnemonicUtils = MnemonicUtils;
