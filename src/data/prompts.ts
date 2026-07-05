import type { RewriteModeId } from "./modes";

export type PromptCategory =
  | "Business"
  | "Writing"
  | "Marketing"
  | "Social Media"
  | "Education"
  | "Programming"
  | "Translation"
  | "Resume"
  | "Email";

export type PromptTemplate = {
  id: string;
  title: string;
  category: PromptCategory;
  mode: RewriteModeId;
  favorite: boolean;
  variables: string[];
  prompt: string;
};

export const promptTemplates: PromptTemplate[] = [
  {
    id: "professional-email",
    title: "Professional Email",
    category: "Email",
    mode: "professional",
    favorite: true,
    variables: ["recipient", "intent"],
    prompt: "Rewrite this message for {recipient} as a clear professional email about {intent}."
  },
  {
    id: "linkedin-post",
    title: "LinkedIn Post",
    category: "Marketing",
    mode: "friendly",
    favorite: true,
    variables: ["audience"],
    prompt: "Rewrite this as a concise LinkedIn update for {audience}."
  },
  {
    id: "github-readme",
    title: "GitHub README",
    category: "Programming",
    mode: "simplify",
    favorite: true,
    variables: ["project"],
    prompt: "Turn this into a clean README section for {project}."
  },
  {
    id: "resume-impact",
    title: "Resume Impact",
    category: "Resume",
    mode: "confident",
    favorite: true,
    variables: ["role"],
    prompt: "Rewrite this as a strong resume bullet for a {role} role."
  },
  {
    id: "summarize-notes",
    title: "Study Notes",
    category: "Education",
    mode: "summarize",
    favorite: false,
    variables: ["words"],
    prompt: "Summarize these notes in about {words} words."
  },
  {
    id: "translate-target",
    title: "Translate",
    category: "Translation",
    mode: "translate",
    favorite: false,
    variables: ["language"],
    prompt: "Translate this text into {language} while preserving the original meaning."
  },
  {
    id: "business-brief",
    title: "Business Brief",
    category: "Business",
    mode: "professional",
    favorite: false,
    variables: ["stakeholder"],
    prompt: "Rewrite this as a concise business brief for {stakeholder}."
  },
  {
    id: "clean-draft",
    title: "Clean Draft",
    category: "Writing",
    mode: "fixGrammar",
    favorite: false,
    variables: [],
    prompt: "Fix grammar, punctuation, and clarity without changing the meaning."
  }
];

const categories: PromptCategory[] = [
  "Business",
  "Programming",
  "Education",
  "Writing",
  "Marketing",
  "Social Media",
  "Translation",
  "Resume",
  "Email"
];

export const promptCategories = categories.filter(
  (category, index, list) => list.indexOf(category) === index
);
