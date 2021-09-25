import 'package:flutter/material.dart';

/// common button with icon & text in center.
class ShadowButton extends StatelessWidget {
  final String text;
  final IconData icon;
  final double width;
  final bool isOn;
  final ColorScheme color;
  final VoidCallback action;

  const ShadowButton({
      Key? key,
      required this.icon,
      required this.text,
      required this.action,
      required this.color,
      this.width = 120.0,
      this.isOn = false,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: action,
      child: Container(
        width: width,
        height: width,
        decoration: BoxDecoration(
          color: isOn ? color.primaryVariant : color.background,
          borderRadius: BorderRadius.circular(10.0),
          boxShadow: [
            if (!isOn)
            BoxShadow(
              color: Color(0x40ADB0BB),
              spreadRadius: 0.0,
              blurRadius: 30.0,
              offset: Offset(0, 10),
            ),
          ],
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, color: color.primary, size: 36.0),
            SizedBox(height: 10.0),
            Text(text, style: TextStyle(fontSize: 14.0)),
          ],
        ),
      )
    );
  }
}
