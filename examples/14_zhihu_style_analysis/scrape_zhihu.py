#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
知乎文章爬取脚本
用于从知乎爬取文章内容

功能特性:
- 完整的浏览器请求头模拟
- 自动重试和错误处理
- 请求频率限制
- Cookie支持
- 详细的日志记录
"""

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry
from bs4 import BeautifulSoup
import json
import time
import random
import os
import re
import logging
from datetime import datetime
from typing import Optional, Dict, List, Any


# 配置日志系统
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)


class ZhihuScraperError(Exception):
    """知乎爬虫基础异常类"""
    pass


class RateLimitError(ZhihuScraperError):
    """请求频率限制异常"""
    pass


class AuthenticationError(ZhihuScraperError):
    """认证失败异常"""
    pass


class NetworkError(ZhihuScraperError):
    """网络连接异常"""
    pass


class ParseError(ZhihuScraperError):
    """解析异常"""
    pass


class ZhihuScraper:
    """知乎文章爬虫类
    
    提供完整的知乎文章爬取功能，包括:
    - 自动重试机制
    - 请求频率限制
    - 完整的错误处理
    - 文章解析和保存
    """
    
    # 默认User-Agent列表（用于轮换避免检测）
    USER_AGENTS = [
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15',
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:123.0) Gecko/20100101 Firefox/123.0',
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:123.0) Gecko/20100101 Firefox/123.0',
    ]
    
    def __init__(self, use_cookie=False, cookie_file=None, raw_cookie=None, rate_limit=2.0, 
                 max_retries=3, timeout=30, rotate_user_agent=False, cookie_format='auto'):
        """
        初始化爬虫实例
        
        Args:
            use_cookie (bool): 是否使用cookie认证
            cookie_file (str): cookie文件路径
            raw_cookie (str): 直接传入浏览器复制的cookie字符串
            rate_limit (float): 请求间隔时间（秒）
            max_retries (int): 最大重试次数
            timeout (int): 请求超时时间（秒）
            rotate_user_agent (bool): 是��轮换User-Agent
            cookie_format (str): cookie格式 ('json', 'netscape', 'header', 'auto')
        """
        self.max_retries = max_retries
        self.timeout = timeout
        self.rotate_user_agent = rotate_user_agent
        self.current_ua_index = 0
        self.cookie_format = cookie_format
        self.authenticated = False  # 认证状态标志
        self.cookie_file = cookie_file  # 保存cookie文件路径以便后续使用
        self.session_start_time = datetime.now()  # 会话开始时间
        
        # 使用最新版本的Chrome User-Agent
        self.headers = {
            'User-Agent': self.USER_AGENTS[0],
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7',
            'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6',
            'Accept-Encoding': 'gzip, deflate, br',
            'Connection': 'keep-alive',
            'Upgrade-Insecure-Requests': '1',
            'Cache-Control': 'max-age=0',
            'sec-ch-ua': '"Chromium";v="122", "Not(A:Brand";v="24", "Google Chrome";v="122"',
            'sec-ch-ua-mobile': '?0',
            'sec-ch-ua-platform': '"macOS"',
            'sec-fetch-dest': 'document',
            'sec-fetch-mode': 'navigate',
            'sec-fetch-site': 'none',
            'sec-fetch-user': '?1',
        }
        
        # 创建带有重试机制的Session
        self.session = self._create_session_with_retry()
        self.session.headers.update(self.headers)
        
        self.success_count = 0  # 成功爬取的文章计数器
        self.fail_count = 0  # 失败计数器
        self.rate_limit = rate_limit  # 请求间隔时间
        self.last_request_time = 0  # 上次请求时间
        
        # 错误统计
        self.error_stats = {
            'timeout': 0,
            'connection': 0,
            'http_4xx': 0,
            'http_5xx': 0,
            'rate_limit': 0,
            'parse': 0,
            'other': 0,
            'authentication': 0
        }
        
        # 加载cookie（优先级：raw_cookie > cookie_file）
        if raw_cookie:
            self._load_raw_cookie(raw_cookie)
        elif use_cookie and cookie_file:
            self._load_cookies(cookie_file, cookie_format)
        
        # 验证cookie是否有效
        if self.session.cookies.get('_zap') or self.session.cookies.get('z_c0'):
            logger.info("检测到认证Cookie，将验证有效性...")
            self._verify_authentication()
        
        logger.info(f"知乎爬虫初始化完成 - 频率限制: {rate_limit}s, 最大重试: {max_retries}, 超时: {timeout}s")
        logger.info(f"认证状态: {'已认证' if self.authenticated else '未认证'}")
    
    def _create_session_with_retry(self) -> requests.Session:
        """
        创建带有自动重试机制的Session
        
        Returns:
            requests.Session: 配置好的Session对象
        """
        session = requests.Session()
        
        # 配置重试策略
        retry_strategy = Retry(
            total=self.max_retries,
            backoff_factor=1,  # 重试间隔指数退避基数
            status_forcelist=[429, 500, 502, 503, 504],  # 需要重试的HTTP状态码
            allowed_methods=["GET", "POST"],  # 允许重试的HTTP方法
            raise_on_status=False,  # 不在重试失败后抛出异常
        )
        
        adapter = HTTPAdapter(max_retries=retry_strategy)
        session.mount("http://", adapter)
        session.mount("https://", adapter)
        
        return session
    
    def _rotate_user_agent(self):
        """
        轮换User-Agent
        """
        if self.rotate_user_agent:
            self.current_ua_index = (self.current_ua_index + 1) % len(self.USER_AGENTS)
            self.headers['User-Agent'] = self.USER_AGENTS[self.current_ua_index]
            self.session.headers.update({'User-Agent': self.headers['User-Agent']})
            logger.debug(f"User-Agent已轮换: {self.headers['User-Agent'][:50]}...")
    
    def _load_raw_cookie(self, raw_cookie):
        """
        从原始cookie字符串加载cookie
        
        支持多种格式:
        - Header格式: "name1=value1; name2=value2; name3=value3"
        - Netscape格式: ".zhihu.com\tTRUE\t/\tFALSE\t0\tname\tvalue"
        
        Args:
            raw_cookie (str): 原始cookie字符串
        """
        try:
            if not raw_cookie or not isinstance(raw_cookie, str):
                logger.warning("提供的cookie字符串为空或格式不正确")
                return
            
            # 自动检测cookie格式
            if '\t' in raw_cookie:
                # Netscape格式
                self._parse_netscape_cookie(raw_cookie)
            elif '=' in raw_cookie:
                # Header格式 (name=value; name2=value2)
                self._parse_header_cookie(raw_cookie)
            else:
                logger.warning("无法识别的cookie格式")
                return
            
            logger.info(f"✓ 已加载原始cookie，共 {len(self.session.cookies)} 个cookie项")
            
        except Exception as e:
            logger.error(f"✗ 加载原始cookie失败: {e}")
            import traceback
            traceback.print_exc()
    
    def _parse_header_cookie(self, cookie_string):
        """
        解析Header格式的cookie字符串
        
        格式: "name1=value1; name2=value2; name3=value3"
        
        Args:
            cookie_string (str): Header格式的cookie字符串
        """
        try:
            cookies = cookie_string.split(';')
            loaded_count = 0
            
            for cookie in cookies:
                cookie = cookie.strip()
                if not cookie or '=' not in cookie:
                    continue
                
                # 分割name和value
                name, value = cookie.split('=', 1)
                name = name.strip()
                value = value.strip()
                
                if name and value:
                    # 设置cookie，默认域名为.zhihu.com
                    self.session.cookies.set(
                        name,
                        value,
                        domain='.zhihu.com',
                        path='/'
                    )
                    loaded_count += 1
            
            logger.info(f"从Header格式解析了 {loaded_count} 个cookie")
            
        except Exception as e:
            logger.error(f"解析Header格式cookie失败: {e}")
            raise
    
    def _parse_netscape_cookie(self, cookie_string):
        """
        解析Netscape格式的cookie字符串
        
        格式: domain\tflag\tpath\tsecure\texpiry\tname\tvalue
        
        Args:
            cookie_string (str): Netscape格式的cookie字符串
        """
        try:
            lines = cookie_string.strip().split('\n')
            loaded_count = 0
            
            for line in lines:
                line = line.strip()
                if not line or line.startswith('#'):
                    continue
                
                parts = line.split('\t')
                if len(parts) >= 7:
                    domain = parts[0]
                    # flag = parts[1]  # TRUE/FALSE
                    path = parts[2]
                    # secure = parts[3]  # TRUE/FALSE
                    # expiry = parts[4]  # 过期时间
                    name = parts[5]
                    value = parts[6]
                    
                    self.session.cookies.set(
                        name,
                        value,
                        domain=domain,
                        path=path
                    )
                    loaded_count += 1
            
            logger.info(f"从Netscape格式解析了 {loaded_count} 个cookie")
            
        except Exception as e:
            logger.error(f"解析Netscape格式cookie失败: {e}")
            raise
    
    def _load_cookies(self, cookie_file, cookie_format='auto'):
        """
        从文件加载cookie
        
        支持多种格式:
        - json: JSON数组格式
        - netscape: Netscape cookies.txt格式
        - header: Header字符串格式
        - auto: 自动检测
        
        Args:
            cookie_file (str): cookie文件路径
            cookie_format (str): cookie格式
        """
        try:
            if not os.path.exists(cookie_file):
                logger.warning(f"⚠ cookie文件不存在: {cookie_file}")
                return
            
            with open(cookie_file, 'r', encoding='utf-8') as f:
                content = f.read().strip()
            
            if not content:
                logger.warning(f"⚠ cookie文件为空: {cookie_file}")
                return
            
            # 自动检测格式
            if cookie_format == 'auto':
                if content.startswith('['):
                    cookie_format = 'json'
                elif '\t' in content:
                    cookie_format = 'netscape'
                elif '=' in content:
                    cookie_format = 'header'
                else:
                    logger.warning("无法自动检测cookie格式")
                    return
            
            # 根据格式解析
            if cookie_format == 'json':
                self._parse_json_cookie_file(content)
            elif cookie_format == 'netscape':
                self._parse_netscape_cookie(content)
            elif cookie_format == 'header':
                self._parse_header_cookie(content)
            else:
                logger.warning(f"不支持的cookie格式: {cookie_format}")
                return
            
            logger.info(f"✓ 已从文件加载cookie: {cookie_file} (格式: {cookie_format})")
            
        except Exception as e:
            logger.error(f"✗ 加载cookie失败: {e}")
            import traceback
            traceback.print_exc()
    
    def _parse_json_cookie_file(self, content):
        """
        解析JSON格式的cookie文件
        
        格式: [{"name": "name1", "value": "value1", "domain": ".zhihu.com"}, ...]
        
        Args:
            content (str): JSON文件内容
        """
        try:
            cookie_data = json.loads(content)
            loaded_count = 0
            
            for cookie in cookie_data:
                name = cookie.get('name')
                value = cookie.get('value')
                domain = cookie.get('domain', '.zhihu.com')
                path = cookie.get('path', '/')
                
                if name and value:
                    self.session.cookies.set(
                        name,
                        value,
                        domain=domain,
                        path=path
                    )
                    loaded_count += 1
            
            logger.info(f"从JSON格式解析了 {loaded_count} 个cookie")
            
        except Exception as e:
            logger.error(f"解析JSON格式cookie失败: {e}")
            raise
    
    def _verify_authentication(self):
        """
        验证cookie认证是否有效
        
        通过访问知乎首页或用户主页来验证认证状态
        """
        try:
            # 检查关键的认证cookie是否存在
            has_auth_cookie = (
                self.session.cookies.get('z_c0') or 
                self.session.cookies.get('_zap') or
                self.session.cookies.get('d_c0')
            )
            
            if not has_auth_cookie:
                logger.info("未检测到认证cookie，将以未认证模式运行")
                self.authenticated = False
                return
            
            # 尝试访问一个需要登录的页面来验证
            test_url = "https://www.zhihu.com/api/v4/me"
            logger.info(f"正在验证认证状态...")
            
            try:
                # 设置API请求头
                api_headers = {
                    'Accept': 'application/json, text/plain, */*',
                    'Referer': 'https://www.zhihu.com/',
                    'x-requested-with': 'fetch',
                }
                
                response = self.session.get(
                    test_url, 
                    headers=api_headers,
                    timeout=10,
                    allow_redirects=False
                )
                
                # 检查响应状态
                if response.status_code == 200:
                    try:
                        data = response.json()
                        if data and not data.get('error'):
                            self.authenticated = True
                            logger.info(f"✓ 认证成功！用户: {data.get('name', '未知')}")
                            return
                    except:
                        pass
                elif response.status_code == 401:
                    logger.warning("⚠ 认证失败：cookie可能已过期")
                    self.authenticated = False
                    return
                    
            except requests.exceptions.RequestException as e:
                logger.warning(f"验证认证时网络请求失败: {e}")
            
            # 如果API验证失败，尝试访问主页
            try:
                homepage_url = "https://www.zhihu.com/"
                response = self.session.get(homepage_url, timeout=10)
                
                # 检查是否重定向到登录页面
                if 'signin' in response.url or response.status_code == 403:
                    logger.warning("⚠ 认证可能已过期，将被重定向到登录页面")
                    self.authenticated = False
                else:
                    # 检查页面内容是否包含登录后的元素
                    if 'SignFlow' not in response.text and 'unlogin' not in response.text:
                        logger.info("✓ 认证状态有效（通过主页验证）")
                        self.authenticated = True
                    else:
                        logger.warning("⚠ 未检测到登录状态")
                        self.authenticated = False
                        
            except requests.exceptions.RequestException as e:
                logger.warning(f"访问主页验证认证时失败: {e}")
                self.authenticated = False
                
        except Exception as e:
            logger.error(f"验证认证时出错: {e}")
            self.authenticated = False
    
    def get_page(self, url, timeout=10, max_retries=3):
        """
        获取网���内容
        
        Args:
            url (str): 目标URL
            timeout (int): 超时时间（秒）
            max_retries (int): 最大重试次数
            
        Returns:
            str: 页面HTML内容，失败返回None
        """
        for attempt in range(max_retries):
            try:
                # 添加随机延迟，避免被反爬
                if attempt > 0:
                    delay = random.uniform(1, 3)
                    time.sleep(delay)
                
                response = self.session.get(url, timeout=timeout)
                response.raise_for_status()
                
                # 检查是否返回了正确的HTML
                if 'text/html' in response.headers.get('Content-Type', ''):
                    return response.text
                else:
                    print(f"警告：返回的内容类型不是HTML: {response.headers.get('Content-Type')}")
                    return response.text
                    
            except requests.exceptions.Timeout:
                print(f"请求超时 (尝试 {attempt + 1}/{max_retries})")
                continue
            except requests.exceptions.ConnectionError:
                print(f"连接错误 (尝试 {attempt + 1}/{max_retries})")
                continue
            except requests.exceptions.HTTPError as e:
                print(f"HTTP错误: {e}")
                if e.response.status_code == 404:
                    return None
                continue
            except Exception as e:
                print(f"未知错误: {e}")
                continue
        
        print(f"获取页面失败，已重试 {max_retries} 次")
        return None
    
    def parse_article(self, html):
        """
        解析知乎文章页面
        
        Args:
            html (str): 页面HTML内容
            
        Returns:
            dict: 包含文章标题、作者、内容、链接等信息的字典
        """
        if not html:
            return None
        
        soup = BeautifulSoup(html, 'html.parser')
        
        article_data = {
            'title': '',
            'author': '',
            'content': '',
            'publish_time': '',
            'url': '',
            'word_count': 0,
            'internal_links': [],
            'external_links': [],
            'images': [],
            'paragraphs': 0,
            'sections': []
        }
        
        try:
            # ========== 提取文章标题 ==========
            title_selectors = [
                'h1.Post-Title',
                'h1.question-title',
                'h1',
                'span.RichText-Title',
                'div.Post-Main h1',
                'div.Post-Header h1'
            ]
            
            for selector in title_selectors:
                title_elem = soup.select_one(selector)
                if title_elem and title_elem.get_text(strip=True):
                    article_data['title'] = title_elem.get_text(strip=True)
                    break
            
            # 如果没有找到标题，尝试从页面<title>标签获取
            if not article_data['title']:
                title_tag = soup.find('title')
                if title_tag:
                    title_text = title_tag.get_text(strip=True)
                    # 移除常见的知乎后缀
                    suffixes_to_remove = [' - 知乎', ' - 专栏文章', ' - 知乎专栏']
                    for suffix in suffixes_to_remove:
                        title_text = title_text.replace(suffix, '')
                    article_data['title'] = title_text.strip()
            
            # ========== 提取作者信息 ==========
            author_selectors = [
                'span.AuthorInfo-name',
                'div.AuthorInfo-content span',
                'a.UserLink-link',
                'span.author-name',
                'div.Post-Header .UserLink',
                '.AuthorInfo-name'
            ]
            
            for selector in author_selectors:
                author_elem = soup.select_one(selector)
                if author_elem:
                    article_data['author'] = author_elem.get_text(strip=True)
                    break
            
            # ========== 提取文章正文内容 ==========
            content_selectors = [
                'div.Post-RichText',
                'div.RichContent-inner',
                'div.ztext.Post-RichText',
                'div.RichText.ztext',
                'div.ContentItem-mainContent',
                'div.QuestionAnswer-content',
                'article',
                'div.content',
                'div.Post-Main'
            ]
            
            content_elem = None
            for selector in content_selectors:
                content_elem = soup.select_one(selector)
                if content_elem:
                    break
            
            if content_elem:
                # 移除不需要的元素，如广告、分享按钮等
                for unwanted in content_elem.select('.ContentItem-actions, .RichText-AD, .Reward'):
                    unwanted.decompose()
                
                # 获取纯文本内容，保留段落结构
                article_data['content'] = content_elem.get_text(separator='\n\n', strip=True)
                
                # ========== 提取链接信息 ==========
                # 提取所有链接
                all_links = content_elem.find_all('a', href=True)
                base_domain = 'zhihu.com'
                
                for link in all_links:
                    href = link['href']
                    link_text = link.get_text(strip=True)
                    
                    # 跳过空链接和锚点链接
                    if not href or href.startswith('#'):
                        continue
                    
                    # 构建完整URL（处理相对路径）
                    if href.startswith('//'):
                        full_url = 'https:' + href
                    elif href.startswith('/'):
                        full_url = 'https://www.zhihu.com' + href
                    elif not href.startswith('http'):
                        full_url = 'https://www.zhihu.com/' + href
                    else:
                        full_url = href
                    
                    # 分类链接
                    if base_domain in full_url:
                        article_data['internal_links'].append({
                            'url': full_url,
                            'text': link_text
                        })
                    else:
                        article_data['external_links'].append({
                            'url': full_url,
                            'text': link_text
                        })
                
                # ========== 提取图片信息 ==========
                images = content_elem.find_all('img', src=True)
                for img in images:
                    src = img['src']
                    alt_text = img.get('alt', '')
                    if src:
                        # 构建完整图片URL
                        if src.startswith('//'):
                            full_img_url = 'https:' + src
                        elif src.startswith('/'):
                            full_img_url = 'https://www.zhihu.com' + src
                        else:
                            full_img_url = src
                        
                        article_data['images'].append({
                            'url': full_img_url,
                            'alt': alt_text
                        })
                
                # ========== 提取段落和小标题信息 ==========
                paragraphs = content_elem.find_all('p')
                article_data['paragraphs'] = len(paragraphs)
                
                # 提取标题层级信息（h2, h3等）
                headings = content_elem.find_all(['h2', 'h3', 'h4'])
                for heading in headings:
                    article_data['sections'].append({
                        'level': heading.name,
                        'text': heading.get_text(strip=True)
                    })
            
            # ========== 如果主内容区域没有找到，尝试更通用的方法 ==========
            if not article_data['content']:
                # 尝试获取所有段落
                paragraphs = soup.find_all('p')
                if paragraphs:
                    article_data['content'] = '\n\n'.join([
                        p.get_text(strip=True) 
                        for p in paragraphs 
                        if p.get_text(strip=True) and len(p.get_text(strip=True)) > 10
                    ])
                    article_data['paragraphs'] = len(paragraphs)
            
            # ========== 计算字数统计 ==========
            if article_data['content']:
                article_data['word_count'] = len(article_data['content'])
                # 计算中文字数
                chinese_chars = len([c for c in article_data['content'] if '\u4e00' <= c <= '\u9fff'])
                article_data['chinese_char_count'] = chinese_chars
                # 计算英文单词数
                english_words = len([w for w in article_data['content'].split() if w.strip()])
                article_data['english_word_count'] = english_words
            
            # ========== 提取发布时间 ==========
            time_selectors = [
                'span.ContentItem-time',
                'div.Post-Header span',
                'time',
                '.Post-Header time',
                'div.Post-Meta time'
            ]
            
            for selector in time_selectors:
                time_elem = soup.select_one(selector)
                if time_elem:
                    article_data['publish_time'] = time_elem.get_text(strip=True)
                    # 尝试提取datetime属性
                    if time_elem.has_attr('datetime'):
                        article_data['publish_time_iso'] = time_elem['datetime']
                    break
            
            # ========== 提取浏览量和点赞数（如果有的话） ==========
            vote_selectors = [
                'span.VoteButton--up',
                'span.HeartButton',
                'div.VoteBar .vote-count'
            ]
            
            for selector in vote_selectors:
                vote_elem = soup.select_one(selector)
                if vote_elem:
                    vote_text = vote_elem.get_text(strip=True)
                    try:
                        article_data['vote_count'] = int(vote_text.replace(',', '').replace('赞同', ''))
                    except ValueError:
                        article_data['vote_count'] = vote_text
                    break
            
        except Exception as e:
            print(f"解析文章时出错: {e}")
            import traceback
            traceback.print_exc()
        
        return article_data
    
    def scrape_article(self, url):
        """
        爬取知乎文章
        
        Args:
            url (str): 知乎文章链接
            
        Returns:
            dict: 文章数据字典
        """
        print(f"开始爬取文章: {url}")
        
        # 获取页面
        html = self.get_page(url)
        
        if not html:
            print("获取页面失败")
            return None
        
        # 解析文章
        article_data = self.parse_article(html)
        article_data['url'] = url
        
        # 输出结果
        if article_data and article_data['content']:
            self.success_count += 1  # 增加成功计数
            print(f"\n{'='*50}")
            print(f"✓ 成功爬取文章 [{self.success_count}]")
            print(f"{'='*50}")
            print(f"标题: {article_data['title']}")
            print(f"作者: {article_data['author']}")
            print(f"发布时间: {article_data['publish_time']}")
            print(f"总字数: {article_data['word_count']} (中文: {article_data.get('chinese_char_count', 0)}, 英文: {article_data.get('english_word_count', 0)})")
            print(f"段落数: {article_data['paragraphs']}")
            print(f"内部链接数: {len(article_data['internal_links'])}")
            print(f"外部链接数: {len(article_data['external_links'])}")
            print(f"图片数量: {len(article_data['images'])}")
            print(f"小标题数: {len(article_data['sections'])}")
            if article_data.get('vote_count'):
                print(f"点赞数: {article_data['vote_count']}")
            print(f"{'='*50}\n")
            print(f"✓ 累计成功爬取文章数量: {self.success_count} 篇\n")
            return article_data
        else:
            self.fail_count += 1  # 增加失败计数
            print("文章解析失败或内容为空")
            return None
    
    def get_user_articles(self, user_url, max_articles=5):
        """
        从用户主页获取文章列表
        
        Args:
            user_url (str): 用户主页URL，例如 https://www.zhihu.com/people/username/posts
            max_articles (int): 最大获取文章数量
            
        Returns:
            list: 文章URL列表
        """
        print(f"开始获取用户文章列表: {user_url}")
        
        # 添加额外的反爬虫措施
        headers = {
            'Referer': 'https://www.zhihu.com/',
            'Origin': 'https://www.zhihu.com',
        }
        self.session.headers.update(headers)
        
        html = self.get_page(user_url)
        if not html:
            print("获取用户主页失败")
            print("提示: 知乎可能需要登录才能访问。请考虑:")
            print("  1. 使用cookie文件进行认证")
            print("  2. 或者直接提供文章URL进行爬取")
            return []
        
        soup = BeautifulSoup(html, 'html.parser')
        article_urls = []
        
        # 尝试多种选择器来获取文章链接
        selectors = [
            'a[class*="ContentItem-title"]',  # 新版知乎
            'a[class*="PostItem-title"]',
            'a[aria-label*="文章"]',
            'div[class*="Post-RichText"] a[href*="/p/"]',
            'a[href*="/p/"]',
        ]
        
        for selector in selectors:
            links = soup.select(selector)
            for link in links:
                href = link.get('href', '')
                # 确保是文章链接
                if '/p/' in href and href not in article_urls:
                    # 构建完整URL
                    if href.startswith('//'):
                        full_url = 'https:' + href
                    elif href.startswith('/'):
                        full_url = 'https://www.zhihu.com' + href
                    else:
                        full_url = href
                    article_urls.append(full_url)
            
            if len(article_urls) >= max_articles:
                break
        
        # 限制返回数量
        article_urls = article_urls[:max_articles]
        
        print(f"找到 {len(article_urls)} 篇文章")
        return article_urls
    
    def scrape_user_articles(self, user_url, max_articles=5, output_dir='workspace/articles'):
        """
        批量爬取用户的文章
        
        Args:
            user_url (str): 用户主页URL
            max_articles (int): 最大爬取文章数量
            output_dir (str): 输出目录
            
        Returns:
            dict: 爬取统计信息 {'total': int, 'success': int, 'failed': int}
        """
        print(f"\n{'='*60}")
        print(f"开始批量爬取用户文章")
        print(f"{'='*60}")
        print(f"用户主页: {user_url}")
        print(f"目标数量: {max_articles} 篇")
        print(f"输出目录: {output_dir}")
        print(f"{'='*60}\n")
        
        # 获取文章列表
        article_urls = self.get_user_articles(user_url, max_articles)
        
        if not article_urls:
            print("未找到文章链接")
            return {'total': 0, 'success': 0, 'failed': 0}
        
        stats = {'total': len(article_urls), 'success': 0, 'failed': 0}
        
        # 批量爬取
        for idx, url in enumerate(article_urls, 1):
            print(f"\n--- 正在爬取第 {idx}/{len(article_urls)} 篇文章 ---")
            
            try:
                article_data = self.scrape_article(url)
                
                if article_data:
                    # 保存文章
                    saved_files = self.save_article(
                        article_data,
                        output_dir=output_dir,
                        save_json=True,
                        save_text=True
                    )
                    
                    print(f"\n保存结果:")
                    print(f"  JSON文件: {saved_files['json']}")
                    print(f"  文本文件: {saved_files['text']}")
                    
                    stats['success'] += 1
                else:
                    print(f"✗ 爬取失败: {url}")
                    stats['failed'] += 1
                
                # 添加延迟避免被封
                if idx < len(article_urls):
                    delay = random.uniform(2, 5)
                    print(f"等待 {delay:.1f} 秒...")
                    time.sleep(delay)
                    
            except Exception as e:
                print(f"✗ 爬取出错: {e}")
                stats['failed'] += 1
                import traceback
                traceback.print_exc()
        
        # 打印最终���计
        print(f"\n{'='*60}")
        print(f"批量爬取完成")
        print(f"{'='*60}")
        print(f"总数: {stats['total']}")
        print(f"成功: {stats['success']}")
        print(f"失败: {stats['failed']}")
        
        # 打印成功摘要
        if stats['success'] > 0:
            self.print_success_summary(output_dir)
        
        # 打印最终报告
        self.print_final_report(output_dir)
        
        return stats
    
    def save_to_json(self, article_data, filepath):
        """
        将文章数据保存为JSON文件
        
        Args:
            article_data (dict): 文章数据
            filepath (str): 保存路径
        """
        try:
            with open(filepath, 'w', encoding='utf-8') as f:
                json.dump(article_data, f, ensure_ascii=False, indent=2)
            print(f"文章已保存到: {filepath}")
        except Exception as e:
            print(f"保存文件时出错: {e}")
    
    def generate_filename(self, title, max_length=100):
        """
        根据文章标题生成安全的文件名
        
        Args:
            title (str): 文章标题
            max_length (int): 文件名最大长度
            
        Returns:
            str: 安全的文件名
        """
        # 移除或替换文件名中的非法字符
        illegal_chars = r'[<>:"/\\|?*]'
        safe_title = re.sub(illegal_chars, '_', title)
        
        # 移除首尾空格和点
        safe_title = safe_title.strip('. ')
        
        # 限制文件名长度
        if len(safe_title) > max_length:
            safe_title = safe_title[:max_length].rsplit('_', 1)[0]
        
        return safe_title if safe_title else 'untitled'
    
    def save_to_text(self, article_data, output_dir='workspace/articles'):
        """
        将文章内容保存为文本文件
        
        Args:
            article_data (dict): 文章数据
            output_dir (str): 输出目录路径
            
        Returns:
            str: 保存的文件路径，失败返回None
        """
        if not article_data:
            print("错误：文章数据为空")
            return None
        
        # 确保输出目录存在
        try:
            os.makedirs(output_dir, exist_ok=True)
        except Exception as e:
            print(f"创建输出目录失败: {e}")
            return None
        
        # 生成文件名
        title = article_data.get('title', '未知标题')
        safe_filename = self.generate_filename(title)
        filepath = os.path.join(output_dir, f"{safe_filename}.txt")
        
        # 处理文件名冲突
        counter = 1
        while os.path.exists(filepath):
            new_filename = f"{safe_filename}_{counter}.txt"
            filepath = os.path.join(output_dir, new_filename)
            counter += 1
        
        try:
            with open(filepath, 'w', encoding='utf-8') as f:
                # 写入文章���信息
                f.write("=" * 70 + "\n")
                f.write(f"标题: {article_data.get('title', '未知')}\n")
                f.write(f"作者: {article_data.get('author', '未知')}\n")
                f.write(f"发布时间: {article_data.get('publish_time', '未知')}\n")
                f.write(f"来源链接: {article_data.get('url', '未知')}\n")
                
                # 写入统计信息
                f.write("-" * 70 + "\n")
                f.write("文章统计:\n")
                f.write(f"  总字数: {article_data.get('word_count', 0)}\n")
                f.write(f"  中文字符: {article_data.get('chinese_char_count', 0)}\n")
                f.write(f"  英文单词: {article_data.get('english_word_count', 0)}\n")
                f.write(f"  段落数: {article_data.get('paragraphs', 0)}\n")
                f.write(f"  小标题数: {len(article_data.get('sections', []))}\n")
                f.write(f"  图片数量: {len(article_data.get('images', []))}\n")
                f.write(f"  内部链接: {len(article_data.get('internal_links', []))}\n")
                f.write(f"  外部链接: {len(article_data.get('external_links', []))}\n")
                if article_data.get('vote_count'):
                    f.write(f"  点赞数: {article_data['vote_count']}\n")
                f.write("-" * 70 + "\n\n")
                
                # 写入小标题目录（如果有）
                sections = article_data.get('sections', [])
                if sections:
                    f.write("目录:\n")
                    for idx, section in enumerate(sections, 1):
                        level = section.get('level', '')
                        text = section.get('text', '')
                        indent = '  ' * (int(level[1]) - 1) if level else ''
                        f.write(f"{indent}{idx}. [{level}] {text}\n")
                    f.write("\n" + "-" * 70 + "\n\n")
                
                # 写入文章正文
                f.write("文章正文:\n")
                f.write("=" * 70 + "\n\n")
                f.write(article_data.get('content', ''))
                f.write("\n\n" + "=" * 70 + "\n")
                
                # 写入内部链接（如果有）
                internal_links = article_data.get('internal_links', [])
                if internal_links:
                    f.write("\n内部链接:\n")
                    for idx, link in enumerate(internal_links, 1):
                        f.write(f"  {idx}. {link.get('text', '')}: {link.get('url', '')}\n")
                
                # 写入外部链接（如果有）
                external_links = article_data.get('external_links', [])
                if external_links:
                    f.write("\n外部链接:\n")
                    for idx, link in enumerate(external_links, 1):
                        f.write(f"  {idx}. {link.get('text', '')}: {link.get('url', '')}\n")
                
                # 写入图片信息（如果有）
                images = article_data.get('images', [])
                if images:
                    f.write("\n图片信息:\n")
                    for idx, img in enumerate(images, 1):
                        alt = img.get('alt', '')
                        url = img.get('url', '')
                        f.write(f"  {idx}. {alt}: {url}\n")
                
                f.write("\n" + "=" * 70 + "\n")
                f.write(f"文件生成时间: {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
                f.write("=" * 70 + "\n")
            
            print(f"✓ 文章已保存为文本文件: {filepath}")
            return filepath
            
        except Exception as e:
            print(f"保存文本文件时出错: {e}")
            import traceback
            traceback.print_exc()
            return None
    
    def save_article(self, article_data, output_dir='workspace/articles', save_json=True, save_text=True, update_index=True):
        """
        保存文章（同时保存JSON和文本文件）
        
        Args:
            article_data (dict): 文章数据
            output_dir (str): 输出目录路径
            save_json (bool): 是否保存JSON文件
            save_text (bool): 是否保存文本文件
            update_index (bool): 是否更新index.json索引文件
            
        Returns:
            dict: 保存的文件路径列表 {'json': path, 'text': path, 'index_updated': bool}
        """
        result = {'json': None, 'text': None, 'index_updated': False}
        
        if not article_data:
            print("错误：文章数据为空")
            return result
        
        # 保存JSON文件
        if save_json:
            title = article_data.get('title', 'untitled')
            safe_filename = self.generate_filename(title)
            json_filepath = os.path.join(output_dir, f"{safe_filename}.json")
            
            # 处理文件名冲突
            counter = 1
            while os.path.exists(json_filepath):
                new_filename = f"{safe_filename}_{counter}.json"
                json_filepath = os.path.join(output_dir, new_filename)
                counter += 1
            
            self.save_to_json(article_data, json_filepath)
            result['json'] = json_filepath
        
        # 保存文本文件
        if save_text:
            text_filepath = self.save_to_text(article_data, output_dir)
            result['text'] = text_filepath
        
        # 更新索引文件
        if update_index:
            index_updated = self.update_index(article_data, output_dir, result)
            result['index_updated'] = index_updated
        
        return result
    
    def get_content_summary(self, article_data):
        """
        获取文章内容摘要信息
        
        Args:
            article_data (dict): 文章数据
            
        Returns:
            str: 摘要信息
        """
        if not article_data:
            return "无文章数据"
        
        summary = f"""
{'='*60}
文章摘要信息
{'='*60}
标题: {article_data.get('title', '未知')}
作者: {article_data.get('author', '未知')}
链接: {article_data.get('url', '未知')}
发布时间: {article_data.get('publish_time', '未知')}
{'-'*60}
内容统计:
  总字数: {article_data.get('word_count', 0)}
  中文字符: {article_data.get('chinese_char_count', 0)}
  英文单词: {article_data.get('english_word_count', 0)}
  段落数: {article_data.get('paragraphs', 0)}
  小标题数: {len(article_data.get('sections', []))}
  图片数量: {len(article_data.get('images', []))}
  内部链接: {len(article_data.get('internal_links', []))}
  外部链接: {len(article_data.get('external_links', []))}
{'-'*60}
点赞数: {article_data.get('vote_count', '未知')}
{'='*60}
"""
        return summary
    
    def get_index_filepath(self, output_dir='workspace/articles'):
        """
        获取index.json文件的路径
        
        Args:
            output_dir (str): 输出目录路径
            
        Returns:
            str: index.json文件的完整路径
        """
        return os.path.join(output_dir, 'index.json')
    
    def load_index(self, output_dir='workspace/articles'):
        """
        加载现有的index.json文件
        
        Args:
            output_dir (str): 输出目录路径
            
        Returns:
            dict: 索引数据字典，如果文件不存在返回空字典
        """
        index_filepath = self.get_index_filepath(output_dir)
        
        if not os.path.exists(index_filepath):
            return {
                'version': '1.0',
                'created_at': '',
                'updated_at': '',
                'total_articles': 0,
                'articles': {}
            }
        
        try:
            with open(index_filepath, 'r', encoding='utf-8') as f:
                index_data = json.load(f)
            
            # 验证索引结构
            if not isinstance(index_data, dict):
                print(f"警告：index.json格式不正确，将创建新的索引")
                return self._create_empty_index()
            
            if 'articles' not in index_data:
                index_data['articles'] = {}
            
            if 'total_articles' not in index_data:
                index_data['total_articles'] = len(index_data.get('articles', {}))
            
            return index_data
            
        except json.JSONDecodeError as e:
            print(f"警告：解析index.json失败: {e}，将创建新的索引")
            return self._create_empty_index()
        except Exception as e:
            print(f"警告：加载index.json失败: {e}，将创建新的索引")
            return self._create_empty_index()
    
    def _create_empty_index(self):
        """
        创建一个空的索引结构
        
        Returns:
            dict: 空的索引数据字典
        """
        return {
            'version': '1.0',
            'created_at': datetime.now().isoformat(),
            'updated_at': datetime.now().isoformat(),
            'total_articles': 0,
            'articles': {}
        }
    
    def update_index(self, article_data, output_dir='workspace/articles', saved_files=None):
        """
        更新index.json文件，记录文章标题和原始链接的映射关系
        
        Args:
            article_data (dict): 文章数据
            output_dir (str): 输出目录路径
            saved_files (dict): 保存的文件路径信息 {'json': path, 'text': path}
            
        Returns:
            bool: 更新成功返回True，失败返回False
        """
        if not article_data or not article_data.get('title'):
            print("错误：文章数据为空或缺少标题")
            return False
        
        # 确保输出目录存在
        try:
            os.makedirs(output_dir, exist_ok=True)
        except Exception as e:
            print(f"创建输出目录失败: {e}")
            return False
        
        # 加载现有索引
        index_data = self.load_index(output_dir)
        
        # 生成唯一的文章ID（使用安全文件名作为ID）
        article_id = self.generate_filename(article_data['title'])
        
        # 检查是否是第一次创建索引
        if not index_data.get('created_at') or index_data['total_articles'] == 0:
            index_data['created_at'] = datetime.now().isoformat()
        
        # 获取原始URL
        original_url = article_data.get('url', '')
        
        # 构建文章条目
        article_entry = {
            'title': article_data['title'],
            'author': article_data.get('author', ''),
            'url': original_url,
            'scraped_at': datetime.now().isoformat(),
            'publish_time': article_data.get('publish_time', ''),
            'word_count': article_data.get('word_count', 0),
            'files': {}
        }
        
        # 添加保存的文件信息
        if saved_files and saved_files.get('json'):
            article_entry['files']['json'] = os.path.basename(saved_files['json'])
        if saved_files and saved_files.get('text'):
            article_entry['files']['text'] = os.path.basename(saved_files['text'])
        
        # 更新索引中的文章条目
        index_data['articles'][article_id] = article_entry
        
        # 更新统计信息
        index_data['total_articles'] = len(index_data['articles'])
        index_data['updated_at'] = datetime.now().isoformat()
        
        # 保存索引文件
        index_filepath = self.get_index_filepath(output_dir)
        try:
            with open(index_filepath, 'w', encoding='utf-8') as f:
                json.dump(index_data, f, ensure_ascii=False, indent=2)
            print(f"✓ 索引已更新: {index_filepath}")
            print(f"  当前文章总数: {index_data['total_articles']}")
            return True
        except Exception as e:
            print(f"保存索引文件时出错: {e}")
            return False
    
    def get_index_info(self, output_dir='workspace/articles'):
        """
        获取索引文件的统计信息
        
        Args:
            output_dir (str): 输出目录路径
            
        Returns:
            dict: 索引统计信息
        """
        index_data = self.load_index(output_dir)
        
        info = {
            'filepath': self.get_index_filepath(output_dir),
            'exists': os.path.exists(self.get_index_filepath(output_dir)),
            'created_at': index_data.get('created_at', '未知'),
            'updated_at': index_data.get('updated_at', '未知'),
            'total_articles': index_data.get('total_articles', 0),
            'version': index_data.get('version', '未知')
        }
        
        return info
    
    def list_all_articles(self, output_dir='workspace/articles'):
        """
        列出索引中的所有文章
        
        Args:
            output_dir (str): 输出目录路径
            
        Returns:
            list: 文章信息列表
        """
        index_data = self.load_index(output_dir)
        articles = []
        
        for article_id, article_info in index_data.get('articles', {}).items():
            articles.append({
                'id': article_id,
                'title': article_info.get('title', ''),
                'url': article_info.get('url', ''),
                'author': article_info.get('author', ''),
                'scraped_at': article_info.get('scraped_at', ''),
                'files': article_info.get('files', {})
            })
        
        return articles
    
    def get_success_count(self):
        """
        获取成功爬取的文章数量
        
        Returns:
            int: 成功爬取的文章数
        """
        return self.success_count
    
    def reset_success_count(self):
        """
        重置成功爬取的文章计数器
        """
        self.success_count = 0
    
    def print_scraping_summary(self, output_dir='workspace/articles'):
        """
        打印爬取统计摘要信息
        
        Args:
            output_dir (str): 输出目录路径
        """
        index_info = self.get_index_info(output_dir)
        session_duration = (datetime.now() - self.session_start_time).total_seconds()
        
        print(f"\n{'='*60}")
        print(f"爬取统计摘要")
        print(f"{'='*60}")
        print(f"本次会话成功爬取: {self.success_count} 篇文章")
        print(f"本次会话失败数量: {self.fail_count} 篇文章")
        print(f"会话持续时间: {session_duration:.1f} 秒")
        if self.success_count > 0:
            avg_time = session_duration / self.success_count
            print(f"平均每篇文章耗时: {avg_time:.1f} 秒")
        print(f"索引文件总文章数: {index_info['total_articles']} 篇")
        print(f"索引文件路径: {index_info['filepath']}")
        if index_info['exists']:
            print(f"最后更新时间: {index_info['updated_at']}")
        print(f"{'='*60}\n")
    
    def print_success_summary(self, output_dir='workspace/articles'):
        """
        打印成功爬取的摘要信息
        
        显示一个醒目的成功消息，包括:
        - 成功爬取的文章数量
        - 索引文件中的文章总数
        - 保存位置信息
        
        Args:
            output_dir (str): 输出目录路径
        """
        index_info = self.get_index_info(output_dir)
        
        # 打印醒目的成功消息
        print(f"\n{'🎉'*30}")
        print(f"{' '*20}爬取任务完成！{' '*20}")
        print(f"{'🎉'*30}\n")
        
        print(f"{'='*60}")
        print(f"✓ 成功爬取文章数量: {self.success_count} 篇")
        print(f"{'='*60}")
        
        if self.success_count > 0:
            print(f"\n📄 本次爬取详情:")
            print(f"   • 成功: {self.success_count} 篇")
            print(f"   • 失败: {self.fail_count} 篇")
            
            # 计算成功率
            total_attempts = self.success_count + self.fail_count
            if total_attempts > 0:
                success_rate = (self.success_count / total_attempts) * 100
                print(f"   • 成功率: {success_rate:.1f}%")
            
            # 显示会话时长
            session_duration = (datetime.now() - self.session_start_time).total_seconds()
            print(f"   • 用时: {session_duration:.1f} 秒")
            if self.success_count > 0:
                avg_time = session_duration / self.success_count
                print(f"   • 平均每篇: {avg_time:.1f} 秒")
        
        print(f"\n📁 文件存储信息:")
        print(f"   • 保存目录: {output_dir}")
        print(f"   • 索引文件: {index_info['filepath']}")
        print(f"   • 索引中总文章数: {index_info['total_articles']} 篇")
        
        if index_info['exists']:
            print(f"   • 最后更新: {index_info['updated_at']}")
        
        # 列出本次爬取的文章
        articles = self.list_all_articles(output_dir)
        if articles and self.success_count > 0:
            print(f"\n📚 本次爬取的文章:")
            for idx, article in enumerate(articles[-self.success_count:], 1):
                print(f"   {idx}. {article['title']}")
                if article.get('author'):
                    print(f"      作者: {article['author']}")
        
        print(f"\n{'='*60}")
        print(f"{'🎉'*30}")
        print()
    
    def print_final_report(self, output_dir='workspace/articles'):
        """
        打印最终报告 - 综合统计信息
        
        Args:
            output_dir (str): 输出目录路径
        """
        print(f"\n{'='*70}")
        print(f"最终报告")
        print(f"{'='*70}")
        
        # 基本统计
        print(f"\n📊 爬取统计:")
        print(f"   ✓ 成功爬取: {self.success_count} 篇文章")
        print(f"   ✗ 失败数量: {self.fail_count} 篇文章")
        
        total = self.success_count + self.fail_count
        if total > 0:
            success_rate = (self.success_count / total) * 100
            print(f"   成功率: {success_rate:.1f}%")
        
        # 时间统计
        session_duration = (datetime.now() - self.session_start_time).total_seconds()
        print(f"\n⏱  时间统计:")
        print(f"   会话时长: {session_duration:.1f} 秒 ({session_duration/60:.1f} 分钟)")
        if self.success_count > 0:
            avg_time = session_duration / self.success_count
            print(f"   平均每篇: {avg_time:.1f} 秒")
        
        # 错误统计
        if any(self.error_stats.values()):
            print(f"\n⚠️  错误统计:")
            for error_type, count in self.error_stats.items():
                if count > 0:
                    print(f"   {error_type}: {count}")
        
        # 索引信息
        index_info = self.get_index_info(output_dir)
        print(f"\n📋 索引信息:")
        print(f"   索引文件: {index_info['filepath']}")
        print(f"   总文章数: {index_info['total_articles']} 篇")
        print(f"   创建时间: {index_info['created_at']}")
        print(f"   更新时间: {index_info['updated_at']}")
        
        print(f"\n{'='*70}\n")
    
    def search_in_index(self, keyword, output_dir='workspace/articles'):
        """
        在索引中搜索文章
        
        Args:
            keyword (str): 搜索关键词
            output_dir (str): 输出目录路径
            
        Returns:
            list: 匹配的文章列表
        """
        articles = self.list_all_articles(output_dir)
        keyword_lower = keyword.lower()
        
        matched = []
        for article in articles:
            title = article['title'].lower()
            author = article['author'].lower()
            url = article['url'].lower()
            
            if (keyword_lower in title or 
                keyword_lower in author or 
                keyword_lower in url):
                matched.append(article)
        
        return matched
    
    def scrape_articles_from_urls(self, article_urls, output_dir='workspace/articles'):
        """
        批量爬取指定的文章URL列表
        
        Args:
            article_urls (list): 文章URL列表
            output_dir (str): 输出目录
            
        Returns:
            dict: 爬取统计信息 {'total': int, 'success': int, 'failed': int}
        """
        print(f"\n{'='*60}")
        print(f"开始批量爬取指定文章")
        print(f"{'='*60}")
        print(f"目标数量: {len(article_urls)} 篇")
        print(f"输出目录: {output_dir}")
        print(f"{'='*60}\n")
        
        if not article_urls:
            print("未提供文章URL")
            return {'total': 0, 'success': 0, 'failed': 0}
        
        stats = {'total': len(article_urls), 'success': 0, 'failed': 0}
        
        # 批量爬取
        for idx, url in enumerate(article_urls, 1):
            print(f"\n--- 正在爬取第 {idx}/{len(article_urls)} 篇文章 ---")
            print(f"URL: {url}")
            
            try:
                article_data = self.scrape_article(url)
                
                if article_data:
                    # 保存文章
                    saved_files = self.save_article(
                        article_data,
                        output_dir=output_dir,
                        save_json=True,
                        save_text=True
                    )
                    
                    print(f"\n保存结果:")
                    print(f"  JSON文件: {saved_files['json']}")
                    print(f"  文本文件: {saved_files['text']}")
                    
                    stats['success'] += 1
                else:
                    print(f"✗ 爬取失败: {url}")
                    stats['failed'] += 1
                
                # 添加延迟避免被封
                if idx < len(article_urls):
                    delay = random.uniform(2, 5)
                    print(f"等待 {delay:.1f} 秒...")
                    time.sleep(delay)
                    
            except Exception as e:
                print(f"✗ 爬取出错: {e}")
                stats['failed'] += 1
                import traceback
                traceback.print_exc()
        
        # 打印最终统计
        print(f"\n{'='*60}")
        print(f"批量爬取完成")
        print(f"{'='*60}")
        print(f"总数: {stats['total']}")
        print(f"成功: {stats['success']}")
        print(f"失败: {stats['failed']}")
        
        # 打印成功摘要
        if stats['success'] > 0:
            self.print_success_summary(output_dir)
        
        # 打印最终报告
        self.print_final_report(output_dir)
        
        return stats


def load_env():
    # Search in current directory and its parents (up to 3 levels)
    current_dir = os.path.dirname(os.path.abspath(__file__))
    for _ in range(3):
        env_path = os.path.join(current_dir, ".env")
        if os.path.exists(env_path):
            with open(env_path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line or line.startswith("#"):
                        continue
                    try:
                        key, value = line.split("=", 1)
                        key = key.strip()
                        value = value.strip().strip('"').strip("'")
                        if key and value:
                            os.environ[key] = value
                    except ValueError:
                        pass
            return
        current_dir = os.path.dirname(current_dir)

def main():
    """主函数 - 批量爬取指定文章URL"""
    import sys
    import os
    
    # 加载环境变量
    load_env()
    
    # 从环境变量读取 Cookie
    cookie = os.environ.get("ZENE_ZHIHU_COOKIE", "")
    
    # 创建爬虫实例，使用raw_cookie参数进行身份认证
    scraper = ZhihuScraper(rotate_user_agent=True, raw_cookie=cookie)
    output_dir = "workspace/articles"
    
    # 预定义的7篇文章URL
    predefined_urls = [
        "https://zhuanlan.zhihu.com/p/659307833",
        "https://zhuanlan.zhihu.com/p/655933730",
        "https://zhuanlan.zhihu.com/p/655754352",
        "https://zhuanlan.zhihu.com/p/648394077",
        "https://zhuanlan.zhihu.com/p/645412104",
        "https://zhuanlan.zhihu.com/p/270576663",
        "https://zhuanlan.zhihu.com/p/111340572"
    ]
    
    # 确定要爬取的URL列表（优先级：命令行参数 > 环境变量 > 预定义URL）
    article_urls = []
    
    if len(sys.argv) > 1:
        # 从命令行参数读取URL
        article_urls = sys.argv[1:]
        print("="*70)
        print("知乎文章批量爬取工具 (命令行模式)")
        print("="*70)
        print(f"从命令行读取到 {len(article_urls)} 个URL")
    else:
        # 使用预定义的URL列表
        article_urls = predefined_urls
        print("="*70)
        print("知乎文章批量爬取工具 (预定义URL模式)")
        print("="*70)
        print(f"使用预定义的 {len(article_urls)} 个URL")
    
    print(f"Cookie状态: {'已加载' if cookie else '未加载'}")
    print(f"认证状态: {'已认证' if scraper.authenticated else '未认证'}")
    print(f"保存目录: {output_dir}")
    print("="*70)
    
    # 显示要爬取的URL列表
    print("\n📋 待爬取的文章URL:")
    for idx, url in enumerate(article_urls, 1):
        print(f"  {idx}. {url}")
    print()
    
    # 执行批量爬取
    stats = scraper.scrape_articles_from_urls(
        article_urls=article_urls,
        output_dir=output_dir
    )
    
    print("\n" + "="*70)
    if stats['success'] > 0:
        print("✓ 任务完成！")
    else:
        print("✗ 没有成功爬取到文章")
        print("\n建议:")
        print("  1. 检查网络连接")
        print("  2. 确保Cookie有效且未过期")
        print("  3. 检查URL是否正确")
    print("="*70)
    
    return stats['success'] > 0


if __name__ == "__main__":
    main()
