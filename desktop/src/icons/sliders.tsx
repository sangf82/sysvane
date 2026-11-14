'use client';

import { motion, type Variants } from 'motion/react';

import {
  getVariants,
  useAnimateIconContext,
  IconWrapper,
  type IconProps,
} from '@animate-ui/react/icons';

type SlidersProps = IconProps<keyof typeof animations>;

const animations = {
  default: {
    path: {
      initial: { x: 0 },
      animate: { x: [0, 1.5, -1.5, 0], transition: { duration: 0.5, ease: 'easeInOut' } },
    },
  } satisfies Record<string, Variants>,
} as const;

function IconComponent({ size, ...props }: SlidersProps) {
  const { controls } = useAnimateIconContext();
  const variants = getVariants(animations);

  return (
    <motion.svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    >
      <motion.path d="M21 4h-7M10 4H3M21 12h-9M8 12H3M21 20h-5M12 20H3M14 2v4M8 10v4M16 18v4" variants={variants.path} initial="initial" animate={controls} />
    </motion.svg>
  );
}

function Sliders(props: SlidersProps) {
  return <IconWrapper icon={IconComponent} {...props} />;
}

export {
  animations,
  Sliders,
  Sliders as SlidersIcon,
  type SlidersProps,
  type SlidersProps as SlidersIconProps,
};
