import 'dart:async';
import 'dart:io' show File;
import 'dart:ui' show ImageByteFormat, ImageFilter;

import 'package:crop/crop.dart';
import 'package:flutter/material.dart';

import 'package:esse/l10n/localizations.dart';
import 'package:esse/utils/pick_image.dart';
import 'package:esse/widgets/shadow_dialog.dart';

class _CropAvatar extends StatefulWidget {
  final Function callback;
  final File image;

  _CropAvatar({Key key, this.callback, this.image}) : super(key: key);

  @override
  _CropAvatarState createState() => _CropAvatarState();
}

class _CropAvatarState extends State<_CropAvatar> {
  CropController _imageController = CropController(aspectRatio: 1.0);
  double _imageScale = 1.0;

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return AlertDialog(
      content: Container(
        height: 180.0,
        //width: 200.0,
        padding: EdgeInsets.only(top: 20.0),
        child: Column(children: [
            Container(
              height: 120.0,
              width: 120.0,
              child: Crop(
                controller: _imageController,
                shape: BoxShape.rectangle,
                helper: Container(
                  decoration: BoxDecoration(
                    //borderRadius: BorderRadius.circular(15.0),
                    border: Border.all(color: color.primary, width: 2),
                  ),
                  child: Icon(Icons.filter_center_focus_rounded,
                    color: color.primary),
                ),
                child: Image(
                  image: FileImage(widget.image), fit: BoxFit.cover)),
            ),
            const SizedBox(height: 8.0),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              mainAxisSize: MainAxisSize.max,
              children: [
                GestureDetector(
                  child: Icon(Icons.zoom_in_rounded,
                    size: 30.0, color: color.primary),
                  onTap: () => setState(() {
                      _imageScale += 0.5;
                      _imageController.scale = _imageScale;
                  }),
                ),
                GestureDetector(
                  child: Icon(Icons.zoom_out_rounded,
                    size: 30.0, color: color.primary),
                  onTap: () => setState(() {
                      if (_imageScale > 1.0) {
                        _imageScale -= 0.5;
                        _imageController.scale = _imageScale;
                      }
                  }),
                ),
            ])
      ])),
      actions: [
        Container(
          margin: const EdgeInsets.only(right: 40.0, bottom: 20.0),
          child: GestureDetector(
            onTap: () => Navigator.of(context).pop(),
            child: Text(lang.cancel))),
        Container(
          margin: const EdgeInsets.only(right: 20.0, bottom: 20.0),
          child: GestureDetector(
            onTap: () async {
              final pixelRatio = MediaQuery.of(context).devicePixelRatio;
              final cropped = await _imageController.crop(pixelRatio: pixelRatio);
              final byteData = await cropped.toByteData(format: ImageByteFormat.png);
              Navigator.of(context).pop();
              widget.callback(byteData.buffer.asUint8List());
            },
            child: Text(lang.ok,
              style: TextStyle(color: color.primary)))),
    ]);
  }
}

Widget _buildMaterialDialogTransitions(
  BuildContext context,
  Animation<double> animation,
  Animation<double> secondaryAnimation,
  Widget child) {
  return BackdropFilter(
    filter: ImageFilter.blur(
      sigmaX: 4 * animation.value, sigmaY: 4 * animation.value),
    child: ScaleTransition(
      scale: CurvedAnimation(
        parent: animation,
        curve: Curves.easeOut,
      ),
      child: child,
  ));
}

void selectAvatar(BuildContext context, Function callback) async {
  final imagePath = await pickImage();
  if (imagePath == null) {
    return;
  }
  final image = File(imagePath);

  showGeneralDialog(
    context: context,
    barrierDismissible: true,
    barrierLabel:
    MaterialLocalizations.of(context).modalBarrierDismissLabel,
    barrierColor: Color(0x26ADB0BB),
    transitionDuration: const Duration(milliseconds: 150),
    transitionBuilder: _buildMaterialDialogTransitions,
    pageBuilder: (BuildContext context, Animation<double> animation,
      Animation<double> secondaryAnimation) {
      return _CropAvatar(callback: callback, image: image);
  });
}
