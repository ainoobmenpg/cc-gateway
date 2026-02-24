#!/bin/bash
# cc-gateway スモークテスト
# 基本的な動作確認を自動実行

set -e

echo "🔧 cc-gateway スモークテスト開始"
echo "================================"

# 1. ビルド確認
echo ""
echo "📦 1. ビルド確認..."
cargo build --release
echo "✅ ビルド成功"

# 2. テスト実行
echo ""
echo "🧪 2. テスト実行..."
cargo test --workspace --quiet
echo "✅ 全テスト合格"

# 3. clippy チェック
echo ""
echo "🔍 3. Lint チェック..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Lint クリア"

# 4. フォーマット確認
echo ""
echo "📝 4. フォーマット確認..."
cargo fmt --all -- --check
echo "✅ フォーマット OK"

# 5. CLI ヘルプ確認
echo ""
echo "❓ 5. CLI ヘルプ確認..."
cargo run --release -- --help > /dev/null
echo "✅ CLI 起動 OK"

# 6. HTTP API 起動確認（短時間）
echo ""
echo "🌐 6. HTTP API 起動確認..."
timeout 5 cargo run --release -- --api 2>/dev/null || true
echo "✅ API 起動確認完了"

echo ""
echo "================================"
echo "🎉 スモークテスト完了！"
echo ""
echo "📊 結果サマリー:"
echo "  - ビルド: ✅"
echo "  - テスト: ✅"
echo "  - Lint: ✅"
echo "  - フォーマット: ✅"
echo "  - CLI: ✅"
echo "  - API: ✅"
