/**
 * 瀑布流高度估算算法 (Updated for new NewsCard.vue)
 * 单位换算参考：Tailwind 默认 1单位 = 4px
 */

const CONSTANTS = {
  // 基础内边距: p-5 (20px*2=40px) + 边框(2px)
  BASE_PADDING: 42,
  
  // 顶部 Label (News Only): text-xs(16px line) + mb-2(8px) + border(2px)
  LABEL_HEIGHT: 26,

  // 图片: h-48(192px) + mb-4(16px) + border(2px)
  IMAGE_HEIGHT: 210,

  // 标题: text-xl/2xl (约24-32px) + mb-1(4px)
  // 移动端 text-xl, 桌面端 text-2xl. 取中间值或偏大值以防重叠
  TITLE_LINE_HEIGHT: 30, 
  TITLE_MARGIN_BOTTOM: 4,
  TITLE_MAX_LINES: 3, // line-clamp-3

  // 副标题: text-sm(20px) + mb-4(16px)
  SUBTITLE_LINE_HEIGHT: 20,
  SUBTITLE_MARGIN_BOTTOM: 16,
  SUBTITLE_MAX_LINES: 2, // line-clamp-2

  // 参与者 (News Only): text-xs(16px) + mb-1(4px)
  // 容器: p-2(16px) + mb-4(16px)
  PARTICIPANT_CONTAINER_PADDING: 32, 
  PARTICIPANT_ROW_HEIGHT: 20,

  // 正文摘要: text-sm + fixed-line-height(1.7) ≈ 24px + mb-4(16px)
  SUMMARY_LINE_HEIGHT: 24,
  SUMMARY_MARGIN_BOTTOM: 16,
  SUMMARY_MAX_LINES: 6, // line-clamp-6

  // 底部 Footer: pt-3(12px) + 内容 + border(1px)
  // Tags (22px) + Meta (16px) + Gap (8px) -> 考虑换行情况，预留约 50-60px
  FOOTER_HEIGHT: 58, 

  // 字符估算 (用于计算行数)
  // 假设卡片宽度约 300px-400px
  CHARS_PER_LINE_TITLE: 14,    // 标题字号大，每行字少
  CHARS_PER_LINE_NORMAL: 26    // 正文/副标题字号小
};

/**
 * 获取卡片预览文本
 * 这里不做截断，截断逻辑在高度计算中处理，以保证视觉一致性
 */
export function getCardPreviewText(article) {
  return article.summary || "";
}

/**
 * 计算卡片“视觉高度”权重
 */
export function getArticleScore(article) {
  let score = CONSTANTS.BASE_PADDING;

  // 1. News 标签
  if (article.type === 'news') {
    score += CONSTANTS.LABEL_HEIGHT;
  }

  // 2. 图片高度
  if (article.image) {
    score += CONSTANTS.IMAGE_HEIGHT;
  }

  // 3. 标题高度 (核心调整：限制最大行数)
  const titleLen = article.title?.length || 0;
  // 计算理论行数
  let titleLines = Math.ceil(titleLen / CONSTANTS.CHARS_PER_LINE_TITLE) || 1;
  // 限制最大行数 (line-clamp-3)
  titleLines = Math.min(titleLines, CONSTANTS.TITLE_MAX_LINES);
  score += (titleLines * CONSTANTS.TITLE_LINE_HEIGHT) + CONSTANTS.TITLE_MARGIN_BOTTOM;

  // 4. 副标题 (限制最大行数)
  if (article.subtitle) {
    const subLen = article.subtitle.length;
    let subLines = Math.ceil(subLen / CONSTANTS.CHARS_PER_LINE_NORMAL) || 1;
    subLines = Math.min(subLines, CONSTANTS.SUBTITLE_MAX_LINES); // line-clamp-2
    score += (subLines * CONSTANTS.SUBTITLE_LINE_HEIGHT) + CONSTANTS.SUBTITLE_MARGIN_BOTTOM;
  }

  // 5. 参与者列表 (News Only)
  if (article.type === "news" && article.participants && article.participants.length > 0) {
    score += CONSTANTS.PARTICIPANT_CONTAINER_PADDING;
    score += article.participants.length * CONSTANTS.PARTICIPANT_ROW_HEIGHT;
  }

  // 6. 正文摘要高度 (核心逻辑)
  const summaryText = getCardPreviewText(article);
  if (summaryText) {
    // 剔除 HTML 标签后计算长度
    const cleanText = summaryText.replace(/<[^>]+>/g, '');
    const textLen = cleanText.length;
    
    // 计算理论行数
    let lines = Math.ceil(textLen / CONSTANTS.CHARS_PER_LINE_NORMAL);
    
    // 强制限制在 line-clamp-6 以内
    // 如果文字很少，按实际行数算；如果文字很多，按6行算
    lines = Math.min(lines, CONSTANTS.SUMMARY_MAX_LINES);
    
    // 如果只有一行文字，也不要加太满，防止 margin 塌陷计算误差
    if (lines > 0) {
        score += (lines * CONSTANTS.SUMMARY_LINE_HEIGHT) + CONSTANTS.SUMMARY_MARGIN_BOTTOM;
    }
  }

  // 7. 底部 Footer (Tags + Author + Date)
  // 虽然 Tags 限制了 slice(0,3)，但为了布局安全，给予固定高度权重
  score += CONSTANTS.FOOTER_HEIGHT;

  return score;
}

/**
 * 构建“按高度分页 + 双列瀑布流”的结构
 */
export function buildMasonryPages(
  articles,
  {
    firstPageLeftOffset = 0,
    pageTargetHeight = 1600, 
  } = {}
) {
  const pages = [];

  let pageIndex = 0;
  let left = [];
  let right = [];
  let leftH = firstPageLeftOffset; 
  let rightH = 0;

  const pushPage = () => {
    pages.push({ left, right });
    pageIndex += 1;
    left = [];
    right = [];
    leftH = 0;
    rightH = 0;
  };

  articles.forEach((article) => {
    const h = getArticleScore(article);

    // 瀑布流贪心算法：永远往短的那一列放
    const putToLeft = leftH <= rightH;
    
    // 预计算放入后的高度
    const nextLeftH = putToLeft ? leftH + h : leftH;
    const nextRightH = putToLeft ? rightH : rightH + h;
    
    const nextMaxH = Math.max(nextLeftH, nextRightH);
    const currentMinH = Math.min(leftH, rightH);

    // 分页判断逻辑：
    // 1. 本页至少有一张卡片
    // 2. 当前页面填充率已经不错了 (>70%)
    // 3. 放入新卡片会导致高度严重溢出
    const shouldBreakPage =
      (left.length + right.length) > 0 && 
      currentMinH > pageTargetHeight * 0.7 &&    
      nextMaxH > pageTargetHeight;

    if (shouldBreakPage) {
      pushPage();
      // 重置后高度归零
      leftH = 0;
      rightH = 0;
      // 在新页面重新判断左右
      if (leftH <= rightH) {
          left.push(article);
          leftH += h;
      } else {
          right.push(article);
          rightH += h;
      }
    } else {
      // 不换页，直接放入
      if (putToLeft) {
        left.push(article);
        leftH += h;
      } else {
        right.push(article);
        rightH += h;
      }
    }
  });

  // 处理剩余卡片
  if (left.length || right.length) {
    pages.push({ left, right });
  }

  return pages;
}

// 兼容旧接口
export function distributeMasonry(articles, initialLeftHeight = 0) {
  const pages = buildMasonryPages(articles, {
    firstPageLeftOffset: initialLeftHeight,
    pageTargetHeight: Infinity, // 无限高度，即不分页
  });
  return pages[0] || { left: [], right: [] };
}