use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use regex::Regex;
use anyhow;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TextLength {
    pub total_articles: usize,
    pub min_length: usize,
    pub max_length: usize,
    pub avg_length: f64,
    pub median_length: usize,
    pub total_chars: usize,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Paragraphs {
    pub total_paragraphs: usize,
    pub avg_paragraphs_per_article: f64,
    pub avg_paragraph_length: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Sentences {
    pub total_sentences: usize,
    pub avg_sentences_per_article: f64,
    pub avg_sentence_length: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CategoryStats {
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Vocabulary {
    pub total_chinese_chars: usize,
    pub unique_words: usize,
    pub total_word_count: usize,
    pub top_words: Vec<(String, usize)>,
    pub top_words_2_3_chars: Vec<(String, usize)>,
    pub top_words_4_plus_chars: Vec<(String, usize)>,
    pub word_categories: HashMap<String, CategoryStats>,
    pub avg_word_length: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PatternSummary {
    pub most_common_pattern: (String, usize),
    pub total_pattern_count: usize,
    pub avg_pattern_per_1000_chars: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Patterns {
    pub counts: HashMap<String, usize>,
    pub density: HashMap<String, f64>,
    pub top_patterns: Vec<(String, usize)>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PatternsReport {
    pub sentence_patterns: Patterns,
    pub top_phrases: Vec<(String, usize)>,
    pub punctuation_usage: HashMap<String, usize>,
    pub pattern_summary: PatternSummary,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ToneReport {
    pub tone_scores: HashMap<String, usize>,
    pub normalized_scores: HashMap<String, f64>,
    pub tone_percentages: HashMap<String, f64>,
    pub tone_distribution: HashMap<String, usize>,
    pub overall_tone: (String, f64),
    pub top_tones: Vec<(String, f64)>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StyleReport {
    pub summary: HashMap<String, String>,
    pub text_length: TextLength,
    pub paragraphs: Paragraphs,
    pub sentences: Sentences,
    pub vocabulary: Vocabulary,
    pub tone: ToneReport,
    pub patterns: PatternsReport,
}

pub struct StyleAnalyzer {
    articles_dir: PathBuf,
    #[allow(dead_code)]
    index_file: PathBuf,
    articles: Vec<Article>,
}

struct Article {
    #[allow(dead_code)]
    file: String,
    content: String,
}

impl StyleAnalyzer {
    pub fn new(articles_dir: &str, index_file: &str) -> Self {
        Self {
            articles_dir: PathBuf::from(articles_dir),
            index_file: PathBuf::from(index_file),
            articles: Vec::new(),
        }
    }

    pub fn load_articles(&mut self) -> anyhow::Result<usize> {
        if !self.articles_dir.exists() {
            return Ok(0);
        }

        let body_re = Regex::new(r"(?s)文章正文:\s*=+_*\s*\n?(.*)")?;
        let footer_markers = ["内部链接:", "图片信息:", "文件生成时间:"];
        let separator = "=".repeat(70);

        for entry in fs::read_dir(&self.articles_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "txt") {
                let content = fs::read_to_string(&path)?;
                let mut clean_content = content.clone();

                if let Some(caps) = body_re.captures(&content) {
                    let mut body = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim().to_string();
                    
                    let mut end_index = body.len();
                    for marker in &footer_markers {
                        if let Some(idx) = body.rfind(marker) {
                            if let Some(sep_idx) = body[..idx].rfind(&separator) {
                                end_index = end_index.min(sep_idx);
                            } else {
                                end_index = end_index.min(idx);
                            }
                        }
                    }
                    body.truncate(end_index);
                    clean_content = body.trim().to_string();
                }

                self.articles.push(Article {
                    file: path.to_string_lossy().to_string(),
                    content: clean_content,
                });
            }
        }

        Ok(self.articles.len())
    }

    pub fn analyze(&self) -> StyleReport {
        let mut report = StyleReport::default();
        report.summary.insert("total_articles".to_string(), self.articles.len().to_string());
        report.summary.insert("articles_directory".to_string(), self.articles_dir.to_string_lossy().to_string());

        if self.articles.is_empty() {
            return report;
        }

        report.text_length = self.analyze_text_length();
        report.paragraphs = self.analyze_paragraphs();
        report.sentences = self.analyze_sentences();
        report.vocabulary = self.analyze_vocabulary();
        report.tone = self.analyze_tone();
        report.patterns = self.analyze_patterns();

        report
    }

    fn analyze_text_length(&self) -> TextLength {
        let lengths: Vec<usize> = self.articles.iter().map(|a| a.content.chars().count()).collect();
        let total_chars: usize = lengths.iter().sum();
        let mut sorted_lengths = lengths.clone();
        sorted_lengths.sort();

        TextLength {
            total_articles: self.articles.len(),
            min_length: *lengths.iter().min().unwrap_or(&0),
            max_length: *lengths.iter().max().unwrap_or(&0),
            avg_length: total_chars as f64 / lengths.len() as f64,
            median_length: sorted_lengths[lengths.len() / 2],
            total_chars,
        }
    }

    fn analyze_paragraphs(&self) -> Paragraphs {
        let mut total_paragraphs = 0;
        let mut paragraph_lengths = Vec::new();

        for article in &self.articles {
            let paras: Vec<&str> = article.content.split('\n').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
            total_paragraphs += paras.len();
            paragraph_lengths.extend(paras.iter().map(|p| p.chars().count()));
        }

        Paragraphs {
            total_paragraphs,
            avg_paragraphs_per_article: total_paragraphs as f64 / self.articles.len() as f64,
            avg_paragraph_length: if paragraph_lengths.is_empty() { 0.0 } else { paragraph_lengths.iter().sum::<usize>() as f64 / paragraph_lengths.len() as f64 },
        }
    }

    fn analyze_sentences(&self) -> Sentences {
        let re = Regex::new(r"[。！？]").unwrap();
        let mut total_sentences = 0;
        let mut sentence_lengths = Vec::new();

        for article in &self.articles {
            let sents: Vec<&str> = re.split(&article.content).map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            total_sentences += sents.len();
            sentence_lengths.extend(sents.iter().map(|s| s.chars().count()));
        }

        Sentences {
            total_sentences,
            avg_sentences_per_article: total_sentences as f64 / self.articles.len() as f64,
            avg_sentence_length: if sentence_lengths.is_empty() { 0.0 } else { sentence_lengths.iter().sum::<usize>() as f64 / sentence_lengths.len() as f64 },
        }
    }

    fn analyze_vocabulary(&self) -> Vocabulary {
        let all_text: String = self.articles.iter().map(|a| a.content.as_str()).collect::<Vec<&str>>().join(" ");
        let char_re = Regex::new(r"[\u{4e00}-\u{9fff}]").unwrap();
        let word_re = Regex::new(r"[\u{4e00}-\u{9fff}]{2,}").unwrap();

        let chinese_chars_count = char_re.find_iter(&all_text).count();
        let words: Vec<String> = word_re.find_iter(&all_text).map(|m| m.as_str().to_string()).collect();
        
        let mut word_freq = HashMap::new();
        for word in &words {
            *word_freq.entry(word.clone()).or_insert(0) += 1;
        }

        let mut top_words: Vec<_> = word_freq.clone().into_iter().collect();
        top_words.sort_by(|a, b| b.1.cmp(&a.1));

        let top_words_2_3: Vec<_> = top_words.iter()
            .filter(|(w, _)| w.chars().count() >= 2 && w.chars().count() <= 3)
            .cloned()
            .take(15)
            .collect();

        let top_words_4_plus: Vec<_> = top_words.iter()
            .filter(|(w, _)| w.chars().count() >= 4)
            .cloned()
            .take(10)
            .collect();

        let conjunctions = vec!["因为", "所以", "但是", "可是", "然而", "不过", "而且", "此外", "另外", "并且", "同时", "于是", "因此", "既然", "如果", "虽然", "尽管", "无论", "不仅", "不但", "既", "又", "或", "或者", "还是", "和", "跟", "与"];
        let pronouns = vec!["我", "你", "他", "她", "它", "我们", "你们", "他们", "她们", "它们", "自己", "本身", "这个", "那个", "这些", "那些", "什么", "哪里", "谁", "这里", "那里", "这么", "那么", "这样", "那样", "如何", "多少"];
        let particles = vec!["的", "了", "着", "过", "吗", "呢", "吧", "啊", "呀", "哦", "嘛"];

        let mut word_categories = HashMap::new();
        
        let conj_count: usize = conjunctions.iter().map(|c| all_text.matches(c).count()).sum();
        word_categories.insert("conjunctions".to_string(), CategoryStats {
            count: conj_count,
            percentage: if words.is_empty() { 0.0 } else { (conj_count as f64 / words.len() as f64 * 100.0).round() / 100.0 * 100.0 },
        });

        let pron_count: usize = pronouns.iter().map(|p| all_text.matches(p).count()).sum();
        word_categories.insert("pronouns".to_string(), CategoryStats {
            count: pron_count,
            percentage: if words.is_empty() { 0.0 } else { (pron_count as f64 / words.len() as f64 * 100.0).round() / 100.0 * 100.0 },
        });

        let part_count: usize = particles.iter().map(|p| all_text.matches(p).count()).sum();
        word_categories.insert("particles".to_string(), CategoryStats {
            count: part_count,
            percentage: if words.is_empty() { 0.0 } else { (part_count as f64 / words.len() as f64 * 100.0).round() / 100.0 * 100.0 },
        });

        Vocabulary {
            total_chinese_chars: chinese_chars_count,
            unique_words: word_freq.len(),
            total_word_count: words.len(),
            top_words: top_words.into_iter().take(20).collect(),
            top_words_2_3_chars: top_words_2_3,
            top_words_4_plus_chars: top_words_4_plus,
            word_categories,
            avg_word_length: if words.is_empty() { 0.0 } else { words.iter().map(|w| w.chars().count()).sum::<usize>() as f64 / words.len() as f64 },
        }
    }

    fn analyze_patterns(&self) -> PatternsReport {
        let all_text: String = self.articles.iter().map(|a| a.content.as_str()).collect::<Vec<&str>>().join(" ");
        let total_chars = all_text.chars().filter(|c| !c.is_whitespace()).count();

        let sentence_patterns = vec![
            ("疑问句", vec![r"吗？", r"呢？", r"吧？", r"什么", r"怎么", r"为什么", r"哪里", r"谁", r"如何", r"怎样"]),
            ("反问句", vec![r"难道.*吗", r"岂能", r"何尝", r"怎么.*能", r"怎么可能", r"岂不是"]),
            ("感叹句", vec![r"！", r"！！", r"啊！", r"太.*了", r"竟然.*！"]),
            ("否定句", vec![r"不", r"没有", r"并非", r"绝不", r"不能", r"不可"]),
        ];

        let mut counts = HashMap::new();
        for (name, patterns) in sentence_patterns {
            let mut count = 0;
            for p in patterns {
                let re = Regex::new(p).unwrap();
                count += re.find_iter(&all_text).count();
            }
            counts.insert(name.to_string(), count);
        }

        let mut density = HashMap::new();
        for (name, count) in &counts {
            density.insert(name.clone(), if total_chars > 0 { (*count as f64 / total_chars as f64 * 1000.0).round() } else { 0.0 });
        }

        let mut top_patterns: Vec<_> = counts.clone().into_iter().collect();
        top_patterns.sort_by(|a, b| b.1.cmp(&a.1));

        let phrase_re = Regex::new(r"[\u{4e00}-\u{9fff}]{4,8}").unwrap();
        let mut phrase_freq = HashMap::new();
        for m in phrase_re.find_iter(&all_text) {
            *phrase_freq.entry(m.as_str().to_string()).or_insert(0) += 1;
        }
        let mut top_phrases: Vec<_> = phrase_freq.into_iter().collect();
        top_phrases.sort_by(|a, b| b.1.cmp(&a.1));

        let mut punctuation_usage = HashMap::new();
        for punct in ["。", "？", "！", "，", "；", "：", "“", "”"] {
            punctuation_usage.insert(punct.to_string(), all_text.matches(punct).count());
        }

        let total_pattern_count: usize = counts.values().sum();
        let most_common = top_patterns.first().cloned().unwrap_or(("无".to_string(), 0));

        PatternsReport {
            sentence_patterns: Patterns {
                counts,
                density,
                top_patterns,
            },
            top_phrases: top_phrases.into_iter().take(15).collect(),
            punctuation_usage,
            pattern_summary: PatternSummary {
                most_common_pattern: most_common,
                total_pattern_count,
                avg_pattern_per_1000_chars: if total_chars > 0 { (total_pattern_count as f64 / total_chars as f64 * 1000.0).round() } else { 0.0 },
            },
        }
    }

    fn analyze_tone(&self) -> ToneReport {
        let mut tone_scores = HashMap::new();
        let keywords = vec![
            ("幽默", vec!["搞笑", "哈哈", "幽默", "逗乐", "有趣", "吐槽", "调侃"]),
            ("严肃", vec!["严肃", "认真", "重要", "必须", "关键", "核心", "责任"]),
            ("讽刺", vec!["讽刺", "嘲讽", "居然", "竟然", "号称", "所谓的", "反讽"]),
        ];

        for (tone, kws) in keywords {
            let mut score = 0;
            for kw in kws {
                for article in &self.articles {
                    score += article.content.matches(kw).count();
                }
            }
            tone_scores.insert(tone.to_string(), score);
        }

        let total_score: usize = tone_scores.values().sum();
        let max_score = *tone_scores.values().max().unwrap_or(&0);

        let mut normalized_scores = HashMap::new();
        let mut tone_percentages = HashMap::new();
        for (tone, score) in &tone_scores {
            normalized_scores.insert(tone.clone(), if max_score > 0 { (*score as f64 / max_score as f64 * 1000.0).round() / 1000.0 } else { 0.0 });
            tone_percentages.insert(tone.clone(), if total_score > 0 { (*score as f64 / total_score as f64 * 100.0).round() } else { 0.0 });
        }

        let mut top_tones: Vec<_> = tone_percentages.clone().into_iter().collect();
        top_tones.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        ToneReport {
            tone_scores,
            normalized_scores,
            tone_percentages,
            tone_distribution: HashMap::new(), // Simplified
            overall_tone: top_tones.first().cloned().unwrap_or(("中性".to_string(), 0.0)),
            top_tones,
        }
    }
}
