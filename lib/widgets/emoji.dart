import 'package:flutter/material.dart';

import 'package:esse/utils/emoji_picker.dart';
import 'package:esse/utils/adaptive.dart';

/// common button with text in center.
class Emoji extends StatelessWidget {
  final Function action;

  const Emoji({
      Key? key,
      required this.action,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    double maxWidth = MediaQuery.of(context).size.width;
    if (isDisplayDesktop(context)) {
      maxWidth -= 520.0;
    }

    return Container(
      height: 156.0,
      child: SingleChildScrollView(
        child: EmojiPicker(
          rows: 3,
          columns: maxWidth ~/ 36,
          maxWidth: maxWidth,
          bgColor: color.background,
          onEmojiSelected: (emoji, category) {
            action(emoji.emoji);
          },
        ),
    ));
  }
}
