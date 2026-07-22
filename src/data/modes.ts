import {
  BadgeCheck,
  FileText,
  Languages,
  PenLine,
  Scissors,
  ShieldCheck,
  Wand2,
  Smile
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

export type RewriteModeId =
  | "fixGrammar"
  | "professional"
  | "friendly"
  | "shorter"
  | "translate"
  | "summarize"
  | "confident"
  | "simplify"
  | "expand";

export type RewriteMode = {
  id: RewriteModeId;
  label: string;
  description: string;
  shortcut?: string;
  icon: LucideIcon;
};

export const rewriteModes: RewriteMode[] = [
  {
    id: "fixGrammar",
    label: "Grammar",
    description: "Clean grammar, tense, punctuation, and sentence flow.",
    shortcut: "Ctrl + 1",
    icon: BadgeCheck
  },
  {
    id: "professional",
    label: "Professional",
    description: "Make the writing polished, clear, and business-ready.",
    shortcut: "Ctrl + 2",
    icon: PenLine
  },
  {
    id: "friendly",
    label: "Friendly",
    description: "Warm up the tone while keeping the meaning intact.",
    icon: Smile
  },
  {
    id: "shorter",
    label: "Shorter",
    description: "Compress the message without losing the core point.",
    icon: Scissors
  },
  {
    id: "translate",
    label: "Translate",
    description: "Translate the text using the chosen target language.",
    icon: Languages
  },
  {
    id: "summarize",
    label: "Summarize",
    description: "Extract the main point into a crisp summary.",
    icon: FileText
  },
  {
    id: "confident",
    label: "Confident",
    description: "Make the message decisive and assertive.",
    icon: ShieldCheck
  },
  {
    id: "simplify",
    label: "Simplify",
    description: "Make the text easier to read and understand.",
    icon: Wand2
  },
  {
    id: "expand",
    label: "Expand",
    description: "Add useful clarity and detail without changing the meaning.",
    icon: FileText
  }
];

export const defaultInput = "";

export const defaultOutput = "";

export const modeLabel = (mode: RewriteModeId) =>
  rewriteModes.find((item) => item.id === mode)?.label ?? "Rewrite";
