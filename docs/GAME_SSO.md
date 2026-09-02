# 游戏统一身份运维说明

主站是唯一身份权威。游戏只通过一次性授权码建立独立 host-only 会话，再申请 60 秒
Ed25519 身份票据；游戏 Node 服务只持有公钥。游客不经过此链路，登录也不会成为游玩前置条件。

## 环境配置

正式主站后端：

```dotenv
HARUHI_GAME_TICKET_PRIVATE_KEY=<32 字节 base64url 种子>
HARUHI_GAME_SSO_REDIRECT_URIS=https://star.haruyuki.cn/auth/callback
```

测试实例必须使用独立 `core.db`，并改为：

```dotenv
PUBLIC_SITE_URL=https://test.haruyuki.cn
HARUHI_GAME_SSO_REDIRECT_URIS=https://test.haruyuki.cn/game/auth/callback
```

禁止在同一测试实例中复用正式身份库，也不要把回调地址配置成通配符。`deploy/gen-secrets.sh`
会生成私钥种子；该值只进入 Rust 后端环境，不能复制给游戏服务或前端。

Rust 后端启动后，从公开 JWKS 读取 Node 所需的公钥：

```bash
curl -fsS https://haruyuki.cn/api/game/jwks | jq -r '.keys[0].x'
```

把输出配置为游戏服务的 `GAME_AUTH_PUBLIC_KEY`，并把 `GAME_AUTH_ISSUERS` 设置为对应
身份实例（正式为 `https://haruyuki.cn`，测试为 `https://test.haruyuki.cn`）。轮换私钥时先发布
主站，再同步公钥并在短窗口内重启游戏 WebSocket 服务；旧票据最长 60 秒后自然失效。

## 暴露边界

- `GET /api/auth/game/authorize`：仅主站登录态可用，签发授权码。
- `/api/game/session*`：游戏站同源代理，建立、查询和注销独立会话。
- `POST /api/game/ticket`：登录用户领取短期票据，需要游戏 CSRF token。
- `GET /api/game/jwks`：公开验签公钥。
- `/uploads/avatars/*`：游戏站只读代理头像。

不要给 `star.haruyuki.cn` 配置 `Domain=.haruyuki.cn` Cookie，也不要把 `core.db`、JWT HMAC
密钥或 Ed25519 私钥交给 Node。当前票据只用于联机连接的可信昵称；排行榜、成就和统计绑定
仍未启用，后续业务数据应放独立 `game.db`。
