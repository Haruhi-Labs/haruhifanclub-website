function esc(s){
  return String(s ?? '').replace(/[<>&]/g, '')
}

function svgArt({ title, subtitle, hue=210 }){
  const h = hue
  const h2 = (h + 40) % 360
  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="900" viewBox="0 0 1200 900">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="hsl(${h} 85% 88%)"/>
      <stop offset="1" stop-color="hsl(${h2} 85% 92%)"/>
    </linearGradient>
    <linearGradient id="stroke" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0" stop-color="hsl(${h} 85% 45%)"/>
      <stop offset="1" stop-color="hsl(${h2} 85% 55%)"/>
    </linearGradient>
    <filter id="soft" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="10" result="b"/>
      <feColorMatrix in="b" type="matrix"
        values="1 0 0 0 0
                0 1 0 0 0
                0 0 1 0 0
                0 0 0 .35 0" result="s"/>
      <feMerge><feMergeNode in="s"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
  </defs>
  <rect x="0" y="0" width="1200" height="900" fill="url(#bg)"/>
  <g opacity=".22">
    <circle cx="180" cy="200" r="150" fill="hsl(${h} 90% 60%)"/>
    <circle cx="980" cy="220" r="180" fill="hsl(${h2} 90% 65%)"/>
    <circle cx="880" cy="720" r="240" fill="hsl(${h} 90% 62%)"/>
    <circle cx="260" cy="720" r="210" fill="hsl(${h2} 90% 66%)"/>
  </g>
  <g filter="url(#soft)">
    <rect x="90" y="90" width="1020" height="720" rx="36" fill="rgba(255,255,255,.72)" stroke="url(#stroke)" stroke-width="8"/>
    <path d="M160 640 C 320 520, 420 520, 560 620 C 700 720, 860 740, 1040 610"
          fill="none" stroke="url(#stroke)" stroke-width="10" stroke-linecap="round" opacity=".65"/>
    <path d="M190 260 C 360 210, 520 210, 660 270 C 820 340, 900 340, 1020 250"
          fill="none" stroke="url(#stroke)" stroke-width="10" stroke-linecap="round" opacity=".40"/>
    <circle cx="300" cy="430" r="56" fill="rgba(255,255,255,.55)" stroke="url(#stroke)" stroke-width="8"/>
    <circle cx="420" cy="460" r="28" fill="rgba(255,255,255,.55)" stroke="url(#stroke)" stroke-width="6"/>
    <circle cx="520" cy="420" r="18" fill="rgba(255,255,255,.55)" stroke="url(#stroke)" stroke-width="5"/>
  </g>
  <g>
    <text x="140" y="190" font-family="ui-sans-serif, -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial"
          font-size="56" font-weight="900" fill="rgba(15,20,28,.92)">${esc(title)}</text>
    <text x="140" y="245" font-family="ui-sans-serif, -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial"
          font-size="28" font-weight="800" fill="rgba(15,20,28,.58)">${esc(subtitle)}</text>
    <text x="140" y="780" font-family="ui-sans-serif, -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial"
          font-size="22" font-weight="800" fill="rgba(15,20,28,.45)">seed demo artwork • svg</text>
  </g>
</svg>`
}

function svgAvatar({ uid, hue=210 }){
  const safe = esc(uid)
  const initials = safe.slice(-2)
  const h = hue
  const h2 = (h + 35) % 360
  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 64 64">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="hsl(${h} 85% 78%)"/>
      <stop offset="1" stop-color="hsl(${h2} 85% 86%)"/>
    </linearGradient>
  </defs>
  <rect x="5" y="5" width="54" height="54" rx="18" fill="url(#g)" stroke="rgba(0,0,0,.10)"/>
  <circle cx="32" cy="30" r="16" fill="rgba(255,255,255,.55)"/>
  <circle cx="26" cy="28" r="2.6" fill="rgba(20,24,32,.82)"/>
  <circle cx="38" cy="28" r="2.6" fill="rgba(20,24,32,.82)"/>
  <path d="M26 37 Q32 42 38 37" fill="none" stroke="rgba(20,24,32,.82)" stroke-width="3" stroke-linecap="round"/>
  <text x="32" y="58" text-anchor="middle"
    font-family="ui-sans-serif, -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial"
    font-size="12" font-weight="900" fill="rgba(20,24,32,.55)">${esc(initials)}</text>
</svg>`
}

function toDataUrl(svg){
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`
}

function iso(ms){ return new Date(ms).toISOString() }

export const seedCreators = [
  { uid:'SOS-0001', avatar_url: toDataUrl(svgAvatar({ uid:'SOS-0001', hue:210 })), created_at: iso(Date.now()-10*864e5) },
  { uid:'SOS-0002', avatar_url: toDataUrl(svgAvatar({ uid:'SOS-0002', hue:285 })), created_at: iso(Date.now()-9*864e5) },
  { uid:'SOS-0003', avatar_url: toDataUrl(svgAvatar({ uid:'SOS-0003', hue:140 })), created_at: iso(Date.now()-8*864e5) },
]

export const seedArtworks = [
  {
    id: -101,
    title:'团长的微笑',
    description:'社团成员的手绘练习稿：以温柔的光感与线条为主。',
    uploader_name:'',
    uploader_uid:'SOS-0001',
    uploader_avatar: seedCreators[0].avatar_url,
    source_type:'personal',
    content_type:'haruhi',
    tags:['凉宫春日','校服','夕阳','手绘'],
    licenses:['可在社交媒体转载','可制作无料周边'],
    origin_url:'',
    created_at: iso(Date.now()-3*864e5),
    reviewed_at: iso(Date.now()-2*864e5),
    review_note:'',
    status:'approved',
    like_total: 7,
    image_url: toDataUrl(svgArt({ title:'团长的微笑', subtitle:'个人作品（凉宫）', hue:210 }))
  },
  {
    id: -102,
    title:'有希的书页',
    description:'偏冷色的构图练习，突出“安静”的气质。',
    uploader_name:'',
    uploader_uid:'SOS-0002',
    uploader_avatar: seedCreators[1].avatar_url,
    source_type:'personal',
    content_type:'haruhi',
    tags:['长门有希','安静','线稿','练习'],
    licenses:['可在社交媒体转载','可在视频或企划中使用'],
    origin_url:'',
    created_at: iso(Date.now()-5*864e5),
    reviewed_at: iso(Date.now()-4*864e5),
    review_note:'',
    status:'approved',
    like_total: 12,
    image_url: toDataUrl(svgArt({ title:'有希的书页', subtitle:'个人作品（凉宫）', hue:270 }))
  },
  {
    id: -103,
    title:'别的番的气氛练习',
    description:'非凉宫内容，但同样是作者的原创练习。',
    uploader_name:'',
    uploader_uid:'SOS-0001',
    uploader_avatar: seedCreators[0].avatar_url,
    source_type:'personal',
    content_type:'other',
    tags:['氛围','原创','练习'],
    licenses:['可在社交媒体转载'],
    origin_url:'',
    created_at: iso(Date.now()-7*864e5),
    reviewed_at: iso(Date.now()-6*864e5),
    review_note:'',
    status:'approved',
    like_total: 3,
    image_url: toDataUrl(svgArt({ title:'气氛练习', subtitle:'个人作品（非凉宫）', hue:140 }))
  },
  {
    id: -104,
    title:'网络搬运：团长海报风',
    description:'模拟“网络图片”条目（用于测试筛选与展示）。',
    uploader_name:'匿名网友',
    uploader_uid:'',
    uploader_avatar:'',
    source_type:'network',
    content_type:'haruhi',
    tags:['海报','凉宫春日','风格参考'],
    licenses:[],
    origin_url:'',
    created_at: iso(Date.now()-1*864e5),
    reviewed_at: iso(Date.now()-1*864e5),
    review_note:'',
    status:'approved',
    like_total: 20,
    image_url: toDataUrl(svgArt({ title:'海报风', subtitle:'网络图片（凉宫）', hue:200 }))
  },
]

export const seedComments = [
  {
    id: -9001,
    artwork_id: -101,
    user_name:'路过的观众',
    avatar_key: 3,
    body:'太好看了！光感很舒服～',
    like_total: 2,
    created_at: iso(Date.now()-1.5*864e5),
  },
  {
    id: -9002,
    artwork_id: -101,
    user_name:'阿虚',
    avatar_key: 8,
    body:'这一张的线条好干净，想学！',
    like_total: 1,
    created_at: iso(Date.now()-1.2*864e5),
  }
]