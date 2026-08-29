from murali_engine import BLUE, GREEN, WHITE, Circle, Label, Rectangle, Scene, Square, Timeline


def build_scene() -> Scene:
    scene = Scene()

    title = scene.add(Label("Hello PyO3", height=0.3, color=WHITE), at=(0.0, 1.0, 0.0))

    circle = Circle(radius=0.5, color=GREEN, segments=32)
    circle.with_stroke(0.03, WHITE)
    moving_circle = scene.add(circle, at=(-1.0, 0.0, 0.0))

    scene.add(Square(size=0.6, color=BLUE), at=(1.0, 0.0, 0.0))
    scene.add(Rectangle(width=1.2, height=0.4, color=WHITE), at=(0.0, -1.0, 0.0))

    timeline = Timeline()
    timeline.animate(title).at(0.0).for_duration(0.4).typewrite_text().spawn()
    timeline.animate(moving_circle).at(0.2).for_duration(0.8).ease("out_cubic").move_to(
        (0.0, -0.2, 0.0)
    ).spawn()
    scene.play(timeline)

    return scene


if __name__ == "__main__":
    scene = build_scene()
    print(scene.tattva_count())
    scene.save_png("rendered_output/python_examples/hello_shapes.png", width=960, fps=1, duration=0.0)
