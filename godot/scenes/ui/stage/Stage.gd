extends MarginContainer

func _ready():
	$SubViewportContainer/SubViewport.size = $SubViewportContainer.size

func _on_ViewportContainer_resized():
	$SubViewportContainer/SubViewport.size = $SubViewportContainer.size

func _on_HSlider_value_changed(value):
	$SubViewportContainer/SubViewport/Camera2D.zoom.x = value
	$SubViewportContainer/SubViewport/Camera2D.zoom.y = value

func _on_Button_pressed():
	$SubViewportContainer/SubViewport/Tree.reset_nodes()
