import 'dart:ui' show ImageFilter;
import 'package:flutter/material.dart';

import 'package:esse/utils/adaptive.dart';

Widget _buildMaterialDialogTransitions(
  BuildContext context,
  Animation<double> animation,
  Animation<double> secondaryAnimation,
  Widget child) {

  return BackdropFilter(
    filter: ImageFilter.blur(sigmaX: 4 * animation.value, sigmaY: 4 * animation.value),
    child: ScaleTransition(
      scale: CurvedAnimation(
        parent: animation,
        curve: Curves.easeOut,
      ),
      child: child,
    )
  );
}

showShadowDialog(BuildContext context, IconData icon, String text, Widget content, [double height=40.0, Widget? right=null]) {
  showGeneralDialog(
    context: context,
    barrierDismissible: true,
    barrierLabel: MaterialLocalizations.of(context).modalBarrierDismissLabel,
    barrierColor: const Color(0x26ADB0BB),
    transitionDuration: const Duration(milliseconds: 150),
    transitionBuilder: _buildMaterialDialogTransitions,
    pageBuilder: (BuildContext context, Animation<double> animation, Animation<double> secondaryAnimation) {
      final color = Theme.of(context).colorScheme;
      final isDesktop = isDisplayDesktop(context);

      return AlertDialog(
        insetPadding: const EdgeInsets.symmetric(horizontal: 24.0, vertical: 40.0),
        title: Stack(
          alignment: Alignment.center,
          children: <Widget>[
            Positioned(
              left: 0.0,
              child: GestureDetector(
                onTap: () => Navigator.of(context).pop(),
                child: Container(
                  width: 20.0,
                  child: Icon(
                    Icons.arrow_back,
                    color: color.primary,
                  )
                ),
              ),
            ),
            if (right != null)
            Positioned(right: 0.0, child: right),
            Container(
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(icon, color: color.primary),
                  const SizedBox(width: 10.0),
                  Text(text, style: const TextStyle(fontSize: 20.0, fontWeight: FontWeight.bold)),
                ]
              )
            )
          ],
        ),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.all(Radius.circular(15.0))
        ),
        content: Container(
          width: 600.0,
          padding: isDesktop ? EdgeInsets.all(height) : const EdgeInsets.all(0.0),
          child: SingleChildScrollView(child: content),
        )
      );
    }
  );
}
