import { useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'

/**
 * Custom hook for handling file uploads with cache management
 */
export function useUploadCache() {
  const queryClient = useQueryClient()

  const clearCache = () => {
    queryClient.clear()
  }

  const invalidateCache = (key: string[]) => {
    queryClient.invalidateQueries({ queryKey: key })
  }

  return {
    clearCache,
    invalidateCache,
  }
}
