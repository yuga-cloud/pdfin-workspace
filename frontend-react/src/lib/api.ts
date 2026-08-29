import { z } from 'zod'

const API_URL = process.env.VITE_API_URL || 'http://localhost:3000'

const ProcessResponseSchema = z.object({
  success: z.boolean(),
  message: z.string(),
  file_size: z.number().optional(),
  error: z.object({
    code: z.string(),
    message: z.string(),
  }).optional(),
})

export type ProcessResponse = z.infer<typeof ProcessResponseSchema>

/**
 * Process a PDF or document file with the specified operation
 * @param file - The file to process
 * @param operation - The operation to perform
 * @returns Processing response
 */
export async function processPdf(file: File, operation: string): Promise<ProcessResponse> {
  const formData = new FormData()
  formData.append('file', file)
  formData.append('operation', operation)

  const response = await fetch(`${API_URL}/api/pdf/${operation}`, {
    method: 'POST',
    body: formData,
  })

  if (!response.ok) {
    const error = await response.json()
    throw new Error(error.error?.message || 'Failed to process file')
  }

  const data = await response.json()
  return ProcessResponseSchema.parse(data)
}

/**
 * Get health status of the API
 */
export async function getHealth(): Promise<{ status: string }> {
  const response = await fetch(`${API_URL}/health`)
  if (!response.ok) {
    throw new Error('API health check failed')
  }
  return response.json()
}
