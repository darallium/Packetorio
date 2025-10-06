shader_type canvas_item;

uniform float spacing = 25.0;
uniform float animation_progress : hint_range(0.0, 1.0) = 0.0;
uniform float dot_size : hint_range(0.0, 1.0) = 1.0;
uniform vec4 dot_color : source_color = vec4(0.0, 0.0, 0.0, 1.0);
uniform bool reverse = false; // ★追加：アニメーションの方向を反転させるフラグ

void fragment() {
    // 1. 進行度がゼロの場合はドットを一切描画しない
    if (animation_progress <= 0.0) {
        discard;
    }

    // 2. 画面のサイズに基づいてグリッドの数を計算
    vec2 screen_size = 1.0 / SCREEN_PIXEL_SIZE;
    vec2 grid_count = floor(screen_size / spacing);

    // 3. UV座標を正規化されたグリッド位置（0.0〜1.0）に変換
    vec2 norm_pos = UV * screen_size / (grid_count * spacing);

    // 4. アニメーションの遅延値を計算
    float delay;
    if (reverse) {
        // ★反転モード：右下(0.0)から左上(1.0)へ
        delay = 1.0 - (norm_pos.x + norm_pos.y) * 0.5;
    } else {
        // 通常モード：左上(0.0)から右下(1.0)へ
        delay = (norm_pos.x + norm_pos.y) * 0.5;
    }

    // 5. 表示判定の仕組み
    // 仕組み1：位置に基づく遅延のしきい値
    float visible_threshold = delay * 0.1 + 0.01;
    if (animation_progress < visible_threshold) {
        discard;
    }

    // 6. ドットのサイズを滑らかに変化させる計算
    float transition = 0.3;
    float scale = smoothstep(
        delay - transition,
        delay + transition,
        animation_progress * (1.0 + transition)
    );

    // 仕組み2：スケール値が極端に小さいものは描画しない
    if (scale < 0.005) {
        discard;
    }

    // 7. ドット（円）を計算
    vec2 grid_center = (floor(UV * screen_size / spacing) + 0.5) * spacing;
    float dist = length(UV * screen_size - grid_center);
    float dot_radius = dot_size * scale * spacing * 0.8;

    // 8. ドットの境界を滑らかにする（アンチエイリアシング）
    float alpha = 1.0 - smoothstep(
        max(dot_radius - 1.5, 0.0),
        dot_radius + 1.5,
        dist
    );

    // 仕組み3：アルファ値が極端に低いものは描画しない
    if (alpha < 0.01) {
		discard;
	}

    COLOR = vec4(dot_color.rgb, alpha);
}