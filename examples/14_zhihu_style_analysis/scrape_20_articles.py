#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Scrape 20 recent articles from Zhihu user profile
"""

import os
import sys
from scrape_zhihu import ZhihuScraper

def main():
    # Configuration
    user_url = "https://www.zhihu.com/people/lipiisme/posts"
    max_articles = 20
    output_dir = "workspace/articles"
    
    # Check for cookie in environment variable
    cookie = os.environ.get("ZENE_ZHIHU_COOKIE", "")
    
    print("="*70)
    print("知乎文章批量爬取工具 - 爬取 20 篇最新文章")
    print("="*70)
    print(f"目标用户: {user_url}")
    print(f"爬取数量: {max_articles} 篇")
    print(f"保存目录: {output_dir}")
    if cookie:
        print(f"认证方式: 使用环境变量中的 Cookie")
    else:
        print(f"认证方式: 未提供 Cookie（可能只能访问公开内容）")
    print("="*70)
    
    # Create scraper instance
    if cookie:
        scraper = ZhihuScraper(
            raw_cookie=cookie,
            rate_limit=2.0,
            max_retries=3,
            rotate_user_agent=True
        )
    else:
        scraper = ZhihuScraper(
            rate_limit=2.0,
            max_retries=3,
            rotate_user_agent=True
        )
    
    # Execute scraping
    stats = scraper.scrape_user_articles(
        user_url=user_url,
        max_articles=max_articles,
        output_dir=output_dir
    )
    
    # Print final summary
    print("\n" + "="*70)
    if stats['success'] > 0:
        print(f"✓ 成功爬取 {stats['success']} 篇文章！")
        print(f"✓ 文章已保存到: {output_dir}")
        
        # List the saved files
        import glob
        json_files = glob.glob(os.path.join(output_dir, "*.json"))
        txt_files = glob.glob(os.path.join(output_dir, "*.txt"))
        
        print(f"\n保存的文件:")
        print(f"  - JSON 文件: {len(json_files)} 个")
        print(f"  - 文本文件: {len(txt_files)} 个")
        print(f"  - 索引文件: {os.path.join(output_dir, 'index.json')}")
    else:
        print("✗ 没有成功爬取到文章")
        print("\n可能的原因:")
        print("  1. 网络连接问题")
        print("  2. 需要登录才能访问用户主页")
        print("  3. 用户主页 URL 不正确")
        print("\n建议:")
        print("  - 设置 ZENE_ZHIHU_COOKIE 环境变量")
        print("  - 检查网络连接")
        print("  - 确认用户主页 URL 是否正确")
    print("="*70)
    
    return stats['success'] > 0

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
