import 'package:flutter/material.dart';

/// common button with text in center.
class ButtonText extends StatelessWidget {
  final String text;
  final double width;
  final double height;
  final bool enable;
  final Function action;

  const ButtonText({
      Key key,
      this.action,
      this.text = null,
      this.width = 450.0,
      this.height = 60.0,
      this.enable = true,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;

    return InkWell(
      onTap: this.enable ? action : () {},
      child: Container(
        width: width,
        height: height,
        decoration: BoxDecoration(
          color: this.enable ? color.primary : Color(0xFFADB0BB),
          borderRadius: BorderRadius.circular(15.0)),
        child: Center(child: Text(text, style: TextStyle(
              fontSize: 20.0,
              color: Colors.white
        ))),
      )
    );
  }
}
