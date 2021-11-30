import torch
import torchvision.models as models

dummy_input = torch.randn((1, 3, 224, 224))
model = models.resnet50()
torch.onnx.export(model, dummy_input, "resnet50.onnx", verbose=True)
