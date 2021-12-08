import 'package:flutter/material.dart';

class DefaultCoreShow extends StatelessWidget {
  final Widget? child;
  const DefaultCoreShow({Key? key, this.child}): super(key: key);

  @override
  Widget build(BuildContext context) {
    final isLight = Theme.of(context).colorScheme.brightness == Brightness.light;

    return Container(
      decoration: BoxDecoration(
        image: DecorationImage(
          image: AssetImage(
            isLight
            ? 'assets/images/background_light.jpg'
            : 'assets/images/background_dark.jpg'
          ),
          fit: BoxFit.cover,
        ),
      ),
      child: Center(child: this.child)
    );
  }
}
