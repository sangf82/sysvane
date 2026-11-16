'use client';

import { motion, type Variants } from 'motion/react';

import {
  getVariants,
  useAnimateIconContext,
  IconWrapper,
  type IconProps,
} from '@animate-ui/react/icons';

type MonitorProps = IconProps<keyof typeof animations>;

const animations = {
  default: {
    rect: {
      initial: { scale: 1 },
      animate: {
        scale: [1, 0.92, 1],
        transition: { duration: 0.4, ease: 'easeInOut' },
      },
    },
    stand: {
      initial: { pathLength: 1, opacity: 1 },
      animate: {
        pathLength: [0, 1],
        opacity: [0, 1],
        transition: { duration: 0.4, ease: 'easeInOut', delay: 0.2 },
      },
    },
  } satisfies Record<string, Variants>,
} as const;

function IconComponent({ size, ...props }: MonitorProps) {
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
      <motion.rect
        width="20"
        height="14"
        x="2"
        y="3"
        rx="2"
        variants={variants.rect}
        initial="initial"
        animate={controls}
      />
      <motion.path
        d="M8 21h8M12 17v4"
        variants={variants.stand}
        initial="initial"
        animate={controls}
      />
    </motion.svg>
  );
}

function Monitor(props: MonitorProps) {
  return <IconWrapper icon={IconComponent} {...props} />;
}

export {
  animations,
  Monitor,
  Monitor as MonitorIcon,
  type MonitorProps,
  type MonitorProps as MonitorIconProps,
};
