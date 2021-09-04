import 'package:flutter/material.dart';

/// common button with text in center.
class ShadowImage extends StatelessWidget {
  final ImageProvider image;
  final double width;

  const ShadowImage({
      Key? key,
      required this.image,
      this.width = 85.0,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      width: width,
      height: width,
      decoration: BoxDecoration(
        boxShadow: [
          BoxShadow(
            color: Color(0xFF2B2E38).withOpacity(0.3),
            spreadRadius: 5.0,
            blurRadius: 15.0,
            offset: Offset(0, 10),
          ),
        ],
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(15.0),
        child: Image(image: image)
      )
    );
  }
}
