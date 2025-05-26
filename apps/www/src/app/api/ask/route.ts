import { createGoogleGenerativeAI } from "@ai-sdk/google";
import { Message, streamText, tool } from "ai";
import { z } from "zod";

interface CodeRegion {
  id: string;
  start: number;
  end: number;
  code: string;
}

const google = createGoogleGenerativeAI({
  apiKey: process.env.GEMINI_API_KEY,
});

const suggestCodeChange = tool({
  description: "Suggest a code change with parameters",
  parameters: z.object({
    startLine: z.number().describe("The starting line of the change"),
    endLine: z.number().describe("The ending line of the change"),
    suggestedCode: z.string().describe("The new code to replace the old code"),
    explanation: z
      .string()
      .describe("Explanation of why this change is suggested"),
  }),
});

const suggestCommentChange = tool({
  description: "Suggest changes to comments or variable/function names",
  parameters: z.object({
    changes: z.array(
      z.object({
        type: z
          .enum(["comment", "rename"])
          .describe(
            "Type of change: 'comment' for comment changes, 'rename' for variable/function renames"
          ),
        startLine: z.number().describe("The starting line of the change"),
        endLine: z.number().describe("The ending line of the change"),
        oldText: z.string().describe("The original text to be changed"),
        newText: z.string().describe("The suggested new text"),
        explanation: z
          .string()
          .describe("Explanation of why this change would improve the code"),
      })
    ),
  }),
});

export async function POST(req: Request) {
  try {
    const { messages } = await req.json();
    const lastMessage = messages[messages.length - 1];
    const codeRegions = lastMessage?.data?.codeRegions || [];

    console.log("Code regions received:", codeRegions);

    const formattedRegions = codeRegions
      .map((region: CodeRegion) => {
        return `Lines ${region.start + 1}-${region.end + 1}:\n\`\`\`\n${
          region.code
        }\n\`\`\``;
      })
      .join("\n\n");

    const systemMessage = `You are a powerful AI coding assistant. You can analyze code and suggest improvements.

When code regions are provided, they will be shown with their content:
Lines <start>-<end>:
\`\`\`
<code content>
\`\`\`

You can use the following tools to propose changes to the code:

1. suggest_code_change: Suggest changes to the code implementation
   - startLine: number (the starting line to change, 1-based)
   - endLine: number (the ending line to change, 1-based)
   - suggestedCode: string (the new code to replace the old code)
   - explanation: string (detailed explanation of the suggested change)

2. suggest_comment_change: Suggest changes to comments or variable/function names
   - changes: array of changes, each containing:
     - type: "comment" | "rename" (type of change)
     - startLine: number (the starting line, 1-based)
     - endLine: number (the ending line, 1-based)
     - oldText: string (the text to be changed)
     - newText: string (the suggested new text)
     - explanation: string (why this change would improve the code)

When suggesting changes:
1. Always reference the specific lines and explain why the change would be beneficial
2. Make sure the suggested changes maintain consistent style with the existing code
3. For renames, consider the full context and ensure the new name better reflects the purpose
4. For comments, focus on clarity, accuracy, and maintaining the codebase's documentation style`;

    const prompt = formattedRegions
      ? `Selected regions:\n${formattedRegions}\n\nUser request: ${lastMessage.content}`
      : lastMessage.content;

    const stream = streamText({
      model: google("gemini-2.0-flash-001"),
      messages: [
        { role: "system", content: systemMessage },
        ...messages.slice(0, -1),
        { role: "user", content: prompt },
      ],
      tools: {
        suggest_code_change: suggestCodeChange,
        suggest_comment_change: suggestCommentChange,
      },
      temperature: 0.7,
      maxTokens: 2048,
    });

    return stream.toDataStreamResponse();
  } catch (error) {
    console.error("Error in POST request:", error);
    return new Response("Internal Server Error", { status: 500 });
  }
}
