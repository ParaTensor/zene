#!/usr/bin/env python3
"""
知乎文章风格分析脚本
读取 workspace/articles/ 目录下的所有文章内容并进行分析
"""

import os
import json
import re
from pathlib import Path
from typing import Dict, List, Any


class StyleAnalyzer:
    """文章风格分析器"""
    
    def __init__(self, articles_dir: str = "workspace/articles", 
                 index_file: str = "workspace/index.json"):
        self.articles_dir = Path(articles_dir)
        self.index_file = Path(index_file)
        self.articles = []
        
    def load_articles(self) -> int:
        """加载所有文章内容"""
        if not self.articles_dir.exists():
            print(f"目录不存在: {self.articles_dir}")
            return 0
            
        txt_files = list(self.articles_dir.glob("*.txt"))
        print(f"找到 {len(txt_files)} 个文章文件")
        
        for txt_file in txt_files:
            try:
                with open(txt_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # 尝试加载对应的JSON文件获取元数据
                json_file = self.articles_dir / txt_file.stem / (txt_file.stem + ".json")
                metadata = {}
                if json_file.exists():
                    try:
                        with open(json_file, 'r', encoding='utf-8') as f:
                            metadata = json.load(f)
                    except Exception as e:
                        print(f"警告: 无法加载元数据文件 {json_file}: {e}")
                
                self.articles.append({
                    'file': str(txt_file),
                    'content': content,
                    'metadata': metadata
                })
                
            except Exception as e:
                print(f"错误: 无法读取文件 {txt_file}: {e}")
        
        print(f"成功加载 {len(self.articles)} 篇文章")
        return len(self.articles)
    
    def load_index(self) -> Dict[str, Any]:
        """加载索引文件"""
        if not self.index_file.exists():
            print(f"索引文件不存在: {self.index_file}")
            return {}
        
        try:
            with open(self.index_file, 'r', encoding='utf-8') as f:
                index = json.load(f)
            print(f"成功加载索引文件，共 {len(index.get('articles', []))} 篇文章")
            return index
        except Exception as e:
            print(f"错误: 无法加载索引文件: {e}")
            return {}
    
    def analyze_text_length(self) -> Dict[str, Any]:
        """分析文章长度分布"""
        if not self.articles:
            return {"error": "没有可分析的文章"}
        
        lengths = [len(article['content']) for article in self.articles]
        
        return {
            "total_articles": len(self.articles),
            "min_length": min(lengths),
            "max_length": max(lengths),
            "avg_length": sum(lengths) / len(lengths),
            "median_length": sorted(lengths)[len(lengths)//2],
            "total_chars": sum(lengths)
        }
    
    def analyze_paragraphs(self) -> Dict[str, Any]:
        """分析段落特征"""
        if not self.articles:
            return {"error": "没有可分析的文章"}
        
        total_paragraphs = 0
        paragraph_lengths = []
        
        for article in self.articles:
            paragraphs = [p.strip() for p in article['content'].split('\n') if p.strip()]
            total_paragraphs += len(paragraphs)
            paragraph_lengths.extend([len(p) for p in paragraphs])
        
        return {
            "total_paragraphs": total_paragraphs,
            "avg_paragraphs_per_article": total_paragraphs / len(self.articles),
            "avg_paragraph_length": sum(paragraph_lengths) / len(paragraph_lengths) if paragraph_lengths else 0
        }
    
    def analyze_sentences(self) -> Dict[str, Any]:
        """分析句子特征"""
        if not self.articles:
            return {"error": "没有可分析的文章"}
        
        total_sentences = 0
        sentence_lengths = []
        
        for article in self.articles:
            # 简单的句子分割（基于句号、问号、感叹号）
            sentences = re.split(r'[。！？]', article['content'])
            sentences = [s.strip() for s in sentences if s.strip()]
            total_sentences += len(sentences)
            sentence_lengths.extend([len(s) for s in sentences])
        
        return {
            "total_sentences": total_sentences,
            "avg_sentences_per_article": total_sentences / len(self.articles),
            "avg_sentence_length": sum(sentence_lengths) / len(sentence_lengths) if sentence_lengths else 0
        }
    
    def analyze_vocabulary(self) -> Dict[str, Any]:
        """分析词汇特征"""
        if not self.articles:
            return {"error": "没有可分析的文章"}
        
        all_text = " ".join([article['content'] for article in self.articles])
        
        # 提取中文字符和词语
        chinese_chars = re.findall(r'[\u4e00-\u9fff]', all_text)
        words = re.findall(r'[\u4e00-\u9fff]{2,}', all_text)
        
        from collections import Counter
        word_freq = Counter(words)
        
        # 分类词汇分析
        # 连接词
        conjunctions = ["因为", "所以", "但是", "可是", "然而", "不过", "而且", "此外", "另外", 
                       "并且", "同时", "于是", "因此", "既然", "如果", "虽然", "尽管", "无论",
                       "不仅", "不但", "既", "又", "或", "或者", "还是", "和", "跟", "与"]
        
        # 代词
        pronouns = ["我", "你", "他", "她", "它", "我们", "你们", "他们", "她们", "它们",
                   "自己", "本身", "这个", "那个", "这些", "那些", "什么", "哪里", "谁",
                   "这里", "那里", "这么", "那么", "这样", "那样", "如何", "多少"]
        
        # 助词
        particles = ["的", "了", "着", "过", "吗", "呢", "吧", "啊", "呀", "哦", "嘛"]
        
        # 统计各类词汇使用频率
        conjunction_count = sum(all_text.count(c) for c in conjunctions)
        pronoun_count = sum(all_text.count(p) for p in pronouns)
        particle_count = sum(all_text.count(p) for p in particles)
        
        # 详细的高频词汇统计（按词长分类）
        word_2_3 = {word: freq for word, freq in word_freq.items() if 2 <= len(word) <= 3}
        word_4_plus = {word: freq for word, freq in word_freq.items() if len(word) >= 4}
        
        return {
            "total_chinese_chars": len(chinese_chars),
            "unique_words": len(set(words)),
            "total_word_count": len(words),
            "top_words": word_freq.most_common(20),
            "top_words_2_3_chars": sorted(word_2_3.items(), key=lambda x: x[1], reverse=True)[:15],
            "top_words_4_plus_chars": sorted(word_4_plus.items(), key=lambda x: x[1], reverse=True)[:10],
            "word_categories": {
                "conjunctions": {
                    "count": conjunction_count,
                    "percentage": round(conjunction_count / len(words) * 100, 2) if words else 0
                },
                "pronouns": {
                    "count": pronoun_count,
                    "percentage": round(pronoun_count / len(words) * 100, 2) if words else 0
                },
                "particles": {
                    "count": particle_count,
                    "percentage": round(particle_count / len(words) * 100, 2) if words else 0
                }
            },
            "avg_word_length": round(sum(len(word) for word in words) / len(words), 2) if words else 0
        }
    
    def analyze_patterns(self) -> Dict[str, Any]:
        """分析句式模式和常用表达"""
        if not self.articles:
            return {"error": "没有可分析的文章"}
        
        all_text = " ".join([article['content'] for article in self.articles])
        
        # 句式模式定义
        sentence_patterns = {
            "疑问句": [
                r"吗？", r"呢？", r"吧？", r"吗\？", r"呢\？", r"吧\？",
                r"什么", r"怎么", r"为什么", r"哪里", r"谁", r"如何", r"怎样",
                r"是不是", r"对不对", r"好吗", r"是吗", r"真的吗"
            ],
            "反问句": [
                r"难道.*吗", r"岂能", r"何尝", r"怎么.*能", r"怎么可能",
                r"哪有.*道理", r"岂不是", r"怎么能", r"怎么会", r"难道不是"
            ],
            "感叹句": [
                r"！", r"！！", r"！！！", r"啊！", r"呀！", r"吧！", r"呢！",
                r"多么", r"真是", r"太.*了", r"好.*啊", r"竟然.*！", r"居然.*！"
            ],
            "否定句": [
                r"不", r"没有", r"没", r"非", r"无", r"并非", r"决不", r"绝不",
                r"毫不", r"从不", r"并非", r"不会", r"不能", r"不可", r"不用"
            ],
            "条件句": [
                r"如果.*就", r"如果.*那么", r"如果.*也", r"只要.*就", r"只有.*才",
                r"除非.*才", r"假如.*就", r"要是.*就", r"一旦.*就", r"倘若.*就"
            ],
            "祈使句": [
                r"请", r"应该", r"必须", r"务必", r"要", r"不要", r"别", r"千万",
                r"一定", r"努力", r"坚持", r"记住", r"不妨", r"最好"
            ],
            "比喻句": [
                r"像.*一样", r"如同", r"仿佛", r"宛如", r"犹如", r"好比", r"类似",
                r"似乎", r"好像", r"恰似", r"像极了", r"活像", r"宛如.*般"
            ],
            "排比句": [
                r"既.*又.*也", r"不.*不.*不", r"又.*又.*又", r"无论.*还是.*都",
                r"既不.*也不", r"或者.*或者.*或者"
            ],
            "因果句": [
                r"因为.*所以", r"由于.*因此", r"既然.*那么", r"因此", r"所以",
                r"导致", r"造成", r"使得", r"由于", r"因为", r"故"
            ],
            "递进句": [
                r"不仅.*而且", r"不但.*还", r"不仅.*还", r"更", r"甚至", r"况且",
                r"何况", r"尤其是", r"更重要的是", r"更进一步"
            ]
        }
        
        # 特殊表达模式
        special_patterns = {
            "强调表达": [
                r"正是", r"确实", r"真正", r"的确", r"务必", r"千万", r"绝对",
                r"完全", r"根本", r"十分", r"极其", r"非常", r"特别", r"格外"
            ],
            "推测表达": [
                r"似乎", r"好像", r"大概", r"可能", r"也许", r"或许", r"恐怕",
                r"估计", r"应该", r"想必", r"未必", r"八成"
            ],
            "否定强调": [
                r"绝不", r"决不", r"根本不", r"完全不", r"丝毫", r"一点.*也不",
                r"绝不.*可能", r"从来.*不", r"从未.*", r"完全不.*"
            ],
            "时间表达": [
                r"现在", r"过去", r"未来", r"今天", r"明天", r"昨天", r"以后",
                r"之前", r"之后", r"同时", r"一直以来", r"始终", r"永远"
            ],
            "数字表达": [
                r"一.*二.*三", r"第一.*第二.*第三", r"首先.*其次.*再次",
                r"一个.*另一个", r"百分之", r"倍", r"次"
            ]
        }
        
        # 统计各类句式模式
        pattern_counts = {name: 0 for name in sentence_patterns.keys()}
        pattern_counts.update({name: 0 for name in special_patterns.keys()})
        
        # 匹配句式模式
        for pattern_type, patterns in sentence_patterns.items():
            for pattern in patterns:
                matches = re.findall(pattern, all_text)
                pattern_counts[pattern_type] += len(matches)
        
        for pattern_type, patterns in special_patterns.items():
            for pattern in patterns:
                matches = re.findall(pattern, all_text)
                pattern_counts[pattern_type] += len(matches)
        
        # 计算句式密度（每千字的句式数量）
        total_chars = len([c for c in all_text if c.strip()])
        pattern_density = {
            name: round(count / total_chars * 1000, 2) if total_chars > 0 else 0
            for name, count in pattern_counts.items()
        }
        
        # 识别常用短语（2-4个词的组合）
        phrases = re.findall(r'[\u4e00-\u9fff]{4,8}', all_text)
        from collections import Counter
        phrase_freq = Counter(phrases)
        
        # 统计标点符号使用
        punctuation_usage = {
            "句号": all_text.count("。"),
            "问号": all_text.count("？"),
            "感叹号": all_text.count("！"),
            "逗号": all_text.count("，"),
            "分号": all_text.count("；"),
            "冒号": all_text.count("："),
            "引号": all_text.count('"') + all_text.count('"') + all_text.count('"') + all_text.count('"'),
            "省略号": all_text.count("…") + all_text.count("……")
        }
        
        return {
            "sentence_patterns": {
                "counts": pattern_counts,
                "density": pattern_density,
                "top_patterns": sorted(pattern_counts.items(), key=lambda x: x[1], reverse=True)[:10]
            },
            "top_phrases": phrase_freq.most_common(15),
            "punctuation_usage": punctuation_usage,
            "pattern_summary": {
                "most_common_pattern": max(pattern_counts.items(), key=lambda x: x[1]) if pattern_counts else ("无", 0),
                "total_pattern_count": sum(pattern_counts.values()),
                "avg_pattern_per_1000_chars": round(sum(pattern_counts.values()) / total_chars * 1000, 2) if total_chars > 0 else 0
            }
        }
    
    def analyze_tone(self) -> Dict[str, Any]:
        """分析语气特征：识别幽默、严肃、讽刺等语气"""
        if not self.articles:
            return {"error": "没有可分析的文章"}
        
        # 语气特征词典
        tone_keywords = {
            "幽默": [
                "搞笑", "哈哈", "笑死", "幽默", "滑稽", "逗乐", "有趣", "好玩", 
                "偷笑", "咯咯", "嘻嘻", "哈哈", "嘿嘿", "哈哈大笑", "捧腹大笑",
                "忍不住笑", "笑翻了", "好笑", "逗我", "笑点", "段子", "梗",
                "吐槽", "调侃", "开黑", "皮一下", "反差", "反差萌", "萌", 
                "萌化", "可爱", "逗趣", "搞笑", "逗比", "沙雕", "欢乐", "欢乐"
            ],
            "严肃": [
                "严肃", "认真", "重要", "必须", "应该", "务必", "关键", 
                "重要", "核心", "重大", "严肃", "重要", "必须", "关键", 
                "核心", "重大", "需要", "必要", "责任", "义务", "要求",
                "规范", "严格", "严谨", "认真", "严肃", "郑重", "正式",
                "正儿八经", "一本正经", "严谨", "严肃", "重要", "关键"
            ],
            "讽刺": [
                "讽刺", "嘲讽", "讽刺意味", "讽刺的是", "居然", "竟然", 
                "号称", "所谓的", "可笑的是", "有趣的是", "讽刺性", 
                "反讽", "讽刺性", "讽刺意味", "嘲弄", "嘲弄", "调侃",
                "挖苦", "揶揄", "讥讽", "讽刺", "讽刺的是", "讽刺意味",
                "自嘲", "讽刺性地", "讽刺地说", "讽刺的是", "讽刺意味"
            ],
            "轻松": [
                "轻松", "愉快", "舒服", "悠闲", "惬意", "放松", "自在",
                "自在", "舒适", "舒服", "轻松", "惬意", "悠闲", "轻松愉快",
                "放松", "自在", "舒适", "惬意", "悠闲", "轻快", "轻轻松松",
                "悠然", "悠然自得", "自在", "舒适", "轻松", "惬意", "悠闲"
            ],
            "愤怒": [
                "愤怒", "生气", "气愤", "愤怒", "愤怒", "愤怒", "气愤",
                "生气", "愤怒", "愤怒", "愤怒", "气愤", "愤怒", "生气",
                "愤怒", "愤怒", "气愤", "生气", "愤怒", "愤怒", "愤怒"
            ],
            "悲伤": [
                "悲伤", "难过", "痛苦", "伤心", "沮丧", "忧郁", "悲伤",
                "难过", "痛苦", "伤心", "沮丧", "忧郁", "悲伤", "难过",
                "痛苦", "伤心", "沮丧", "忧郁", "悲伤", "难过", "痛苦"
            ],
            "赞赏": [
                "赞赏", "赞扬", "赞美", "夸奖", "表扬", "优秀", "出色",
                "精彩", "卓越", "杰出", "优秀", "出色", "精彩", "卓越",
                "杰出", "优秀", "出色", "精彩", "卓越", "杰出", "优秀"
            ],
            "质疑": [
                "质疑", "怀疑", "疑问", "疑问", "不解", "困惑", "疑惑",
                "质疑", "怀疑", "疑问", "不解", "困惑", "疑惑", "质疑",
                "怀疑", "疑问", "不解", "困惑", "疑惑", "质疑", "怀疑"
            ]
        }
        
        # 标点符号语气特征
        punctuation_tone = {
            "感叹": r"！{2,}",  # 多个感叹号表示强烈情感
            "疑问": r"？{2,}",  # 多个问号表示强烈疑问
            "省略": r"…{2,}",  # 多个省略号表示无语或含蓄
        }
        
        # 统计各类语气特征
        tone_scores = {tone: 0 for tone in tone_keywords.keys()}
        tone_scores.update({tone: 0 for tone in punctuation_tone.keys()})
        
        # 每篇文章的语气分析
        article_tones = []
        
        for article in self.articles:
            content = article['content']
            article_tone = {tone: 0 for tone in tone_keywords.keys()}
            
            # 基于关键词的语气分析
            for tone, keywords in tone_keywords.items():
                for keyword in keywords:
                    count = content.count(keyword)
                    if count > 0:
                        tone_scores[tone] += count
                        article_tone[tone] += count
            
            # 基于标点符号的语气分析
            for tone, pattern in punctuation_tone.items():
                matches = re.findall(pattern, content)
                if matches:
                    tone_scores[tone] += len(matches)
            
            article_tones.append(article_tone)
        
        # 计算每篇文章的主导语气
        dominant_tones = []
        for article_tone in article_tones:
            if sum(article_tone.values()) > 0:
                dominant = max(article_tone.items(), key=lambda x: x[1])
                dominant_tones.append(dominant)
            else:
                dominant_tones.append(("中性", 0))
        
        # 统计各类主导语气出现的次数
        tone_distribution = {}
        for tone, score in dominant_tones:
            tone_distribution[tone] = tone_distribution.get(tone, 0) + 1
        
        # 计算语气强度（标准化）
        max_score = max(tone_scores.values()) if max(tone_scores.values()) > 0 else 1
        normalized_scores = {tone: round(score / max_score, 3) if max_score > 0 else 0 
                           for tone, score in tone_scores.items()}
        
        # 计算整体语气倾向
        total_tone_score = sum(tone_scores.values())
        tone_percentages = {
            tone: round(score / total_tone_score * 100, 2) if total_tone_score > 0 else 0
            for tone, score in tone_scores.items()
        }
        
        return {
            "tone_scores": tone_scores,
            "normalized_scores": normalized_scores,
            "tone_percentages": tone_percentages,
            "tone_distribution": tone_distribution,
            "dominant_tones": dominant_tones,
            "overall_tone": max(tone_percentages.items(), key=lambda x: x[1]) if total_tone_score > 0 else ("中性", 0),
            "top_tones": sorted(tone_percentages.items(), key=lambda x: x[1], reverse=True)[:5]
        }
    
    def generate_report(self) -> Dict[str, Any]:
        """生成综合分析报告"""
        report = {
            "summary": {
                "total_articles": len(self.articles),
                "articles_directory": str(self.articles_dir),
                "index_file": str(self.index_file)
            },
            "text_length": self.analyze_text_length(),
            "paragraphs": self.analyze_paragraphs(),
            "sentences": self.analyze_sentences(),
            "vocabulary": self.analyze_vocabulary(),
            "tone": self.analyze_tone(),
            "patterns": self.analyze_patterns()
        }
        
        return report
    
    def print_report(self, report: Dict[str, Any]):
        """打印分析报告"""
        print("\n" + "="*60)
        print("知乎文章风格分析报告")
        print("="*60)
        
        print("\n【摘要信息】")
        summary = report.get('summary', {})
        print(f"文章总数: {summary.get('total_articles', 0)}")
        print(f"文章目录: {summary.get('articles_directory', '')}")
        
        print("\n【文本长度分析】")
        length = report.get('text_length', {})
        if 'error' not in length:
            print(f"最短: {length.get('min_length', 0)} 字符")
            print(f"最长: {length.get('max_length', 0)} 字符")
            print(f"平均: {length.get('avg_length', 0):.1f} 字符")
            print(f"中位数: {length.get('median_length', 0)} 字符")
            print(f"总字符数: {length.get('total_chars', 0)}")
        else:
            print(length['error'])
        
        print("\n【段落分析】")
        paragraphs = report.get('paragraphs', {})
        if 'error' not in paragraphs:
            print(f"总段落数: {paragraphs.get('total_paragraphs', 0)}")
            print(f"平均每篇文章段落数: {paragraphs.get('avg_paragraphs_per_article', 0):.1f}")
            print(f"平均段落长度: {paragraphs.get('avg_paragraph_length', 0):.1f} 字符")
        else:
            print(paragraphs['error'])
        
        print("\n【句子分析】")
        sentences = report.get('sentences', {})
        if 'error' not in sentences:
            print(f"总句子数: {sentences.get('total_sentences', 0)}")
            print(f"平均每篇文章句子数: {sentences.get('avg_sentences_per_article', 0):.1f}")
            print(f"平均句子长度: {sentences.get('avg_sentence_length', 0):.1f} 字符")
        else:
            print(sentences['error'])
        
        print("\n【词汇分析】")
        vocabulary = report.get('vocabulary', {})
        if 'error' not in vocabulary:
            print(f"中文字符总数: {vocabulary.get('total_chinese_chars', 0)}")
            print(f"不重复词汇数: {vocabulary.get('unique_words', 0)}")
            print(f"高频词 Top 10:")
            for word, freq in vocabulary.get('top_words', [])[:10]:
                print(f"  {word}: {freq}次")
        else:
            print(vocabulary['error'])
        
        print("\n【语气分析】")
        tone = report.get('tone', {})
        if 'error' not in tone:
            overall = tone.get('overall_tone', ('未知', 0))
            print(f"整体语气倾向: {overall[0]} ({overall[1]:.1f}%)")
            
            print(f"\n语气分布 Top 5:")
            for tone_name, percentage in tone.get('top_tones', [])[:5]:
                if percentage > 0:
                    print(f"  {tone_name}: {percentage:.1f}%")
            
            print(f"\n各类语气得分:")
            for tone_name, score in tone.get('tone_scores', {}).items():
                if score > 0:
                    print(f"  {tone_name}: {score}次")
            
            print(f"\n文章主导语气统计:")
            for tone_name, count in tone.get('tone_distribution', {}).items():
                print(f"  {tone_name}: {count}篇")
        else:
            print(tone['error'])
        
        print("\n【词汇和句式分析】")
        patterns = report.get('patterns', {})
        if 'error' not in patterns:
            summary = patterns.get('pattern_summary', {})
            print(f"最常用句式: {summary.get('most_common_pattern', ('无', 0))[0]} ({summary.get('most_common_pattern', ('无', 0))[1]}次)")
            print(f"总句式模式数: {summary.get('total_pattern_count', 0)}")
            print(f"每千字句式密度: {summary.get('avg_pattern_per_1000_chars', 0):.1f}")
            
            print(f"\n句式模式 Top 8:")
            for pattern_name, count in patterns.get('sentence_patterns', {}).get('top_patterns', [])[:8]:
                if count > 0:
                    print(f"  {pattern_name}: {count}次")
            
            print(f"\n常用短语 Top 10:")
            for phrase, freq in patterns.get('top_phrases', [])[:10]:
                print(f"  {phrase}: {freq}次")
            
            print(f"\n标点符号使用:")
            punct = patterns.get('punctuation_usage', {})
            for punct_name, count in punct.items():
                if count > 0:
                    print(f"  {punct_name}: {count}个")
        else:
            print(patterns['error'])
        
        print("\n" + "="*60)
    
    def save_report(self, report: Dict[str, Any], output_file: str = "workspace/style_report.json"):
        """保存分析报告到JSON文件"""
        output_path = Path(output_file)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(report, f, ensure_ascii=False, indent=2)
        
        print(f"\n分析报告已保存到: {output_path}")


def main():
    """主函数"""
    print("="*60)
    print("知乎文章风格分析工具")
    print("="*60)
    
    # 创建分析器实例
    analyzer = StyleAnalyzer()
    
    # 加载索引文件
    index = analyzer.load_index()
    if index:
        print(f"索引中的文章数: {len(index.get('articles', []))}")
    
    # 加载文章内容
    article_count = analyzer.load_articles()
    
    if article_count == 0:
        print("\n警告: 没有找到文章文件，无法进行风格分析")
        print("请先运行爬虫脚本爬取文章")
        return
    
    # 生成分析报告
    report = analyzer.generate_report()
    
    # 打印报告
    analyzer.print_report(report)
    
    # 保存报告
    analyzer.save_report(report)
    
    print("\n分析完成！")


if __name__ == "__main__":
    main()
